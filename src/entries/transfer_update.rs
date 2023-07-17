use crate::ckb::indexer::get_cota_smt_root;
use crate::ckb::rpc::get_withdraw_info;
use crate::entries::helper::{
    generate_claim_key, generate_claim_value, generate_withdrawal_key_v1,
    generate_withdrawal_value_v1, with_lock,
};
use crate::entries::smt::{generate_history_smt, init_smt};
use crate::entries::witness::parse_witness_withdraw_proof;
use crate::models::claim::is_exist_in_claim;
use crate::models::withdrawal::nft::{get_withdrawal_cota_by_lock_hash, WithdrawDb};
use crate::request::transfer::{TransferUpdate, TransferUpdateReq};
use crate::smt::transaction::store_transaction::StoreTransaction;
use crate::smt::RootSaver;
use crate::utils::error::Error;
use crate::ROCKS_DB;
use cota_smt::common::*;
use cota_smt::molecule::prelude::*;
use cota_smt::smt::{blake2b_256, H256};
use cota_smt::transfer_update::{
    TransferUpdateCotaNFTV2Entries, TransferUpdateCotaNFTV2EntriesBuilder,
};
use log::error;
use molecule::hex_string;

pub async fn generate_transfer_update_smt(
    transfer_update_req: TransferUpdateReq,
) -> Result<(H256, TransferUpdateCotaNFTV2Entries, H256), Error> {
    let transfers = transfer_update_req.transfers;
    let transfer_lock_script = transfer_update_req.lock_script;
    let transfers_len = transfers.len();
    if transfers_len == 0 {
        return Err(Error::RequestParamNotFound("transfers".to_string()));
    }
    let cota_id_index_pairs: Vec<([u8; 20], [u8; 4])> = transfers
        .iter()
        .map(|transfer| (transfer.cota_id, transfer.token_index))
        .collect();
    let withdrawal_lock_hash = blake2b_256(&transfer_update_req.withdrawal_lock_script);
    let sender_withdrawals =
        get_withdrawal_cota_by_lock_hash(withdrawal_lock_hash, &cota_id_index_pairs)?.0;
    if sender_withdrawals.is_empty() || sender_withdrawals.len() != transfers_len {
        return Err(Error::CotaIdAndTokenIndexHasNotWithdrawn);
    }
    let is_receiver = sender_withdrawals
        .iter()
        .any(|withdrawal| &withdrawal.receiver_lock_script == &transfer_lock_script);
    if !is_receiver {
        return Err(Error::CotaIdAndTokenIndexHasNotWithdrawn);
    }
    let transfer_lock_hash = blake2b_256(&transfer_lock_script);
    let is_claimed = sender_withdrawals.iter().any(|withdrawal| {
        is_exist_in_claim(
            transfer_lock_hash,
            withdrawal.cota_id,
            withdrawal.token_index,
            withdrawal.out_point,
        )
    });
    if is_claimed {
        return Err(Error::CotaIdAndTokenIndexHasNotWithdrawn);
    }
    let withdrawal_block_number = sender_withdrawals.first().unwrap().block_number;
    let withdrawal_tx_hash = sender_withdrawals.first().unwrap().tx_hash;
    if sender_withdrawals[1..]
        .iter()
        .any(|withdrawal| withdrawal.block_number != withdrawal_block_number)
    {
        return Err(Error::WithdrawCotaNFTsNotInOneTx);
    }

    let mut action_vec: Vec<u8> = Vec::new();
    if transfers_len == 1 {
        action_vec.extend("Transfer the NFT ".as_bytes());
        action_vec.extend(hex_string(&sender_withdrawals.first().unwrap().cota_id).as_bytes());
        action_vec.extend(hex_string(&sender_withdrawals.first().unwrap().token_index).as_bytes());
        action_vec.extend(" to ".as_bytes());
        action_vec.extend(hex_string(&transfers.first().unwrap().to_lock_script).as_bytes());
        action_vec.extend(" and update the NFT information".as_bytes());
    }
    let action_bytes = BytesBuilder::default()
        .set(action_vec.iter().map(|v| Byte::from(*v)).collect())
        .build();

    let mut claimed_keys: Vec<ClaimCotaNFTKey> = Vec::new();
    let mut claimed_values: Vec<Byte32> = Vec::new();
    let mut claimed_infos: Vec<ClaimCotaNFTInfo> = Vec::new();
    let mut withdrawal_keys: Vec<WithdrawalCotaNFTKeyV1> = Vec::new();
    let mut withdrawal_values: Vec<WithdrawalCotaNFTValueV1> = Vec::new();
    let mut transfer_update_leaves: Vec<(H256, H256)> = Vec::with_capacity(transfers_len * 2);
    let mut previous_leaves: Vec<(H256, H256)> = Vec::with_capacity(transfers_len * 2);
    for (withdrawal_db, transfer) in sender_withdrawals.into_iter().zip(transfers.clone()) {
        let WithdrawDb {
            cota_id,
            token_index,
            characteristic,
            state,
            configure,
            out_point,
            version,
            ..
        } = withdrawal_db;
        let nft_info = CotaNFTInfoBuilder::default()
            .characteristic(Characteristic::from_slice(&characteristic).unwrap())
            .configure(Byte::from(configure))
            .state(Byte::from(state))
            .build();
        let claimed_info = ClaimCotaNFTInfoBuilder::default()
            .nft_info(nft_info)
            .version(Byte::from(version))
            .build();
        claimed_infos.push(claimed_info);

        let TransferUpdate { to_lock_script, .. } = transfer;

        let (withdrawal_key, key) = generate_withdrawal_key_v1(
            cota_id,
            token_index,
            transfer_update_req.transfer_out_point,
        );
        let (withdrawal_value, value) = generate_withdrawal_value_v1(
            configure,
            transfer.state,
            transfer.characteristic,
            &to_lock_script,
        );
        withdrawal_keys.push(withdrawal_key);
        withdrawal_values.push(withdrawal_value);
        transfer_update_leaves.push((key, value));
        previous_leaves.push((key, H256::zero()));

        let (claimed_key, key) = generate_claim_key(cota_id, token_index, out_point);
        claimed_keys.push(claimed_key);

        let (claimed_value, value) = generate_claim_value(version);
        claimed_values.push(claimed_value);
        transfer_update_leaves.push((key, value));
        previous_leaves.push((key, H256::zero()));
    }

    let transfer_smt_root = get_cota_smt_root(&transfer_lock_script).await?;

    let transaction = &StoreTransaction::new(ROCKS_DB.transaction());
    let transfer_lock_hash = blake2b_256(&transfer_lock_script);
    let mut transfer_update_smt = init_smt(transaction, transfer_lock_hash)?;
    // Add lock to transfer smt
    with_lock(transfer_lock_hash, || {
        generate_history_smt(
            &mut transfer_update_smt,
            transfer_lock_hash,
            transfer_smt_root,
        )?;
        transfer_update_smt
            .update_all(transfer_update_leaves.clone())
            .map_err(|e| Error::SMTError(e.to_string()))?;
        transfer_update_smt.save_root_and_leaves(previous_leaves.clone())?;
        transfer_update_smt.commit()
    })?;

    let leaf_keys: Vec<H256> = transfer_update_leaves.iter().map(|leave| leave.0).collect();
    let transfer_update_merkle_proof = transfer_update_smt
        .merkle_proof(leaf_keys.clone())
        .map_err(|e| {
            error!("Transfer update SMT proof error: {:?}", e.to_string());
            Error::SMTProofError("Transfer".to_string())
        })?;
    let transfer_update_merkle_proof_compiled = transfer_update_merkle_proof
        .compile(leaf_keys)
        .map_err(|e| {
            error!("Transfer SMT proof error: {:?}", e.to_string());
            Error::SMTProofError("Transfer update".to_string())
        })?;

    let transfer_update_merkel_proof_vec: Vec<u8> = transfer_update_merkle_proof_compiled.into();
    let transfer_update_merkel_proof_bytes = BytesBuilder::default()
        .extend(
            transfer_update_merkel_proof_vec
                .iter()
                .map(|v| Byte::from(*v)),
        )
        .build();

    let withdraw_info = get_withdraw_info(
        withdrawal_block_number,
        withdrawal_lock_hash,
        withdrawal_tx_hash,
    )
    .await?;
    let withdraw_proof = parse_witness_withdraw_proof(
        withdraw_info.witnesses,
        &cota_id_index_pairs,
        withdraw_info.block_number,
    )?;

    let transfer_update_entries = TransferUpdateCotaNFTV2EntriesBuilder::default()
        .claim_keys(
            ClaimCotaNFTKeyVecBuilder::default()
                .set(claimed_keys)
                .build(),
        )
        .claim_infos(
            ClaimCotaNFTInfoVecBuilder::default()
                .set(claimed_infos)
                .build(),
        )
        .withdrawal_keys(
            WithdrawalCotaNFTKeyV1VecBuilder::default()
                .set(withdrawal_keys)
                .build(),
        )
        .withdrawal_values(
            WithdrawalCotaNFTValueV1VecBuilder::default()
                .set(withdrawal_values)
                .build(),
        )
        .proof(transfer_update_merkel_proof_bytes)
        .action(action_bytes)
        .withdrawal_proof(withdraw_proof)
        .leaf_keys(Byte32Vec::default())
        .leaf_values(Byte32Vec::default())
        .raw_tx(withdraw_info.raw_tx)
        .output_index(withdraw_info.output_index)
        .tx_proof(withdraw_info.tx_proof)
        .build();

    Ok((
        *transfer_update_smt.root(),
        transfer_update_entries,
        withdraw_info.block_hash,
    ))
}
