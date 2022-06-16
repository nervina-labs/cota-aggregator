use crate::ckb::indexer::get_cota_smt_root;
use crate::ckb::rpc::get_withdraw_info;
use crate::entries::helper::{
    generate_claim_key, generate_claim_value, generate_withdrawal_key_v1,
    generate_withdrawal_value_v1, with_lock,
};
use crate::entries::smt::{generate_history_smt, init_smt};
use crate::entries::witness::parse_withdraw_witness;
use crate::models::withdrawal::{get_withdrawal_cota_by_lock_hash, WithdrawDb};
use crate::request::transfer::TransferReq;
use crate::request::withdrawal::TransferWithdrawal;
use crate::smt::db::db::RocksDB;
use crate::smt::transaction::store_transaction::StoreTransaction;
use crate::smt::RootSaver;
use crate::utils::error::Error;
use crate::utils::helper::diff_time;
use chrono::prelude::*;
use cota_smt::common::*;
use cota_smt::molecule::prelude::*;
use cota_smt::smt::{blake2b_256, H256};
use cota_smt::transfer::{TransferCotaNFTV2Entries, TransferCotaNFTV2EntriesBuilder};
use log::error;
use molecule::hex_string;

pub async fn generate_transfer_smt(
    db: &RocksDB,
    transfer_req: TransferReq,
) -> Result<(H256, TransferCotaNFTV2Entries, H256), Error> {
    let transfers = transfer_req.transfers;
    let transfers_len = transfers.len();
    if transfers_len == 0 {
        return Err(Error::RequestParamNotFound("transfers".to_string()));
    }
    let cota_id_index_pairs: Vec<([u8; 20], [u8; 4])> = transfers
        .iter()
        .map(|transfer| (transfer.cota_id, transfer.token_index))
        .collect();
    let withdraw_lock_hash = blake2b_256(&transfer_req.withdrawal_lock_script);
    let sender_withdrawals =
        get_withdrawal_cota_by_lock_hash(withdraw_lock_hash, &cota_id_index_pairs)?.0;
    if sender_withdrawals.is_empty() || sender_withdrawals.len() != transfers_len {
        return Err(Error::CotaIdAndTokenIndexHasNotWithdrawn);
    }
    let withdrawal_block_number = sender_withdrawals.first().unwrap().block_number;
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
    }
    let action_bytes = BytesBuilder::default()
        .set(action_vec.iter().map(|v| Byte::from(*v)).collect())
        .build();

    let mut claimed_keys: Vec<ClaimCotaNFTKey> = Vec::new();
    let mut claimed_values: Vec<Byte32> = Vec::new();
    let mut withdrawal_keys: Vec<WithdrawalCotaNFTKeyV1> = Vec::new();
    let mut withdrawal_values: Vec<WithdrawalCotaNFTValueV1> = Vec::new();
    let mut transfer_update_leaves: Vec<(H256, H256)> = Vec::with_capacity(transfers_len * 2);
    let mut previous_leaves: Vec<(H256, H256)> = Vec::with_capacity(transfers_len * 2);
    let start_time = Local::now().timestamp_millis();
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
        let TransferWithdrawal { to_lock_script, .. } = transfer;

        let (withdrawal_key, key) =
            generate_withdrawal_key_v1(cota_id, token_index, transfer_req.transfer_out_point);
        let (withdrawal_value, value) =
            generate_withdrawal_value_v1(configure, state, characteristic, &to_lock_script);
        withdrawal_keys.push(withdrawal_key);
        withdrawal_values.push(withdrawal_value);
        transfer_update_leaves.push((key, value));
        previous_leaves.push((key, H256::zero()));

        let (claimed_key, key) = generate_claim_key(cota_id, token_index, out_point);
        let (claimed_value, value) = generate_claim_value(version);
        claimed_keys.push(claimed_key);
        claimed_values.push(claimed_value);
        transfer_update_leaves.push((key, value));
        previous_leaves.push((key, H256::zero()));
    }
    diff_time(
        start_time,
        "Generate transfer smt object with update leaves",
    );

    let transfer_smt_root = get_cota_smt_root(&transfer_req.lock_script).await?;

    let transaction = &StoreTransaction::new(db.transaction());
    let transfer_lock_hash = blake2b_256(&transfer_req.lock_script);
    let mut transfer_smt = init_smt(transaction, transfer_lock_hash)?;
    // Add lock to transfer smt
    with_lock(transfer_lock_hash, || {
        generate_history_smt(&mut transfer_smt, transfer_lock_hash, transfer_smt_root)?;
        transfer_smt
            .update_all(transfer_update_leaves.clone())
            .map_err(|e| Error::SMTError(e.to_string()))?;
        transfer_smt.save_root_and_leaves(previous_leaves.clone())?;
        transfer_smt.commit()
    })?;

    let start_time = Local::now().timestamp_millis();
    let transfer_merkle_proof = transfer_smt
        .merkle_proof(transfer_update_leaves.iter().map(|leave| leave.0).collect())
        .map_err(|e| {
            error!("Transfer SMT proof error: {:?}", e.to_string());
            Error::SMTProofError("Transfer".to_string())
        })?;
    let transfer_merkle_proof_compiled = transfer_merkle_proof
        .compile(transfer_update_leaves.clone())
        .map_err(|e| {
            error!("Transfer SMT proof error: {:?}", e.to_string());
            Error::SMTProofError("Transfer".to_string())
        })?;
    diff_time(start_time, "Generate transfer smt proof");

    let transfer_merkel_proof_vec: Vec<u8> = transfer_merkle_proof_compiled.into();
    let transfer_merkel_proof_bytes = BytesBuilder::default()
        .extend(transfer_merkel_proof_vec.iter().map(|v| Byte::from(*v)))
        .build();

    let withdraw_info =
        get_withdraw_info(withdrawal_block_number, transfer_req.withdrawal_lock_script).await?;
    let withdraw_leaf_proof = parse_withdraw_witness(
        withdraw_info.witnesses,
        &cota_id_index_pairs,
        withdraw_info.block_number,
    )?;

    let transfer_entries = TransferCotaNFTV2EntriesBuilder::default()
        .claim_keys(
            ClaimCotaNFTKeyVecBuilder::default()
                .set(claimed_keys)
                .build(),
        )
        .claim_values(
            ClaimCotaNFTValueVecBuilder::default()
                .set(claimed_values)
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
        .proof(transfer_merkel_proof_bytes)
        .action(action_bytes)
        .withdrawal_proof(withdraw_leaf_proof.withdraw_proof)
        .leaf_keys(
            Byte32VecBuilder::default()
                .set(withdraw_leaf_proof.leaf_keys)
                .build(),
        )
        .leaf_values(
            Byte32VecBuilder::default()
                .set(withdraw_leaf_proof.leaf_values)
                .build(),
        )
        .raw_tx(withdraw_info.raw_tx)
        .output_index(withdraw_info.output_index)
        .tx_proof(withdraw_info.tx_proof)
        .build();

    Ok((
        *transfer_smt.root(),
        transfer_entries,
        withdraw_info.block_hash,
    ))
}
