use super::helper::generate_subkey_key;
use crate::ckb::indexer::get_cota_smt_root;
use crate::ckb::rpc::get_withdraw_info;
use crate::entries::helper::{
    generate_claim_key, generate_claim_value, generate_withdrawal_key_v1,
    generate_withdrawal_value_v1, with_lock,
};
use crate::entries::smt::{generate_history_smt, init_smt};
use crate::entries::witness::parse_witness_withdraw_proof;
use crate::models::claim::is_exist_in_claim;
use crate::models::extension::subkey::get_subkey_by_pubkey_hash;
use crate::models::withdrawal::nft::{get_withdrawal_cota_by_lock_hash, WithdrawDb};
use crate::request::transfer::{SequentTransfer, SequentTransferReq, SubKeyUnlock};
use crate::smt::transaction::store_transaction::StoreTransaction;
use crate::smt::{CotaSMT, RootSaver};
use crate::utils::error::Error;
use crate::ROCKS_DB;
use cota_smt::common::*;
use cota_smt::molecule::prelude::*;
use cota_smt::smt::{blake2b_256, H256};
use cota_smt::transfer::{TransferCotaNFTV2Entries, TransferCotaNFTV2EntriesBuilder};
use joyid_smt::joyid::{SubKeyUnlockEntries, SubKeyUnlockEntriesBuilder};
use molecule::hex_string;

pub type SequentTransferResult = (
    H256,
    TransferCotaNFTV2Entries,
    Option<SubKeyUnlockEntries>,
    H256,
);

pub async fn generate_sequent_transfer_smt(
    transfer_req: SequentTransferReq,
) -> Result<SequentTransferResult, Error> {
    let transfers = transfer_req.transfers;
    let current_transfer = transfers.last().unwrap();
    let transfer_lock_script = transfer_req.lock_script;
    let subkey_opt = transfer_req.subkey;
    let transfer_lock_hash = blake2b_256(&transfer_lock_script);
    let transfers_len = transfers.len();
    if transfers_len == 0 {
        return Err(Error::RequestParamNotFound("transfers".to_string()));
    }
    let mut sender_withdrawals = Vec::with_capacity(transfers_len);
    let mut current_withdrawal = WithdrawDb::default();
    for (index, transfer) in transfers.clone().into_iter().enumerate() {
        let withdrawals = get_withdrawal_cota_by_lock_hash(transfer.withdrawal_lock_hash, &[(
            transfer.cota_id,
            transfer.token_index,
        )])?
        .0;
        if withdrawals.is_empty() {
            return Err(Error::CotaIdAndTokenIndexHasNotWithdrawn);
        }
        sender_withdrawals.push(withdrawals.first().unwrap().clone());

        if index == transfers_len - 1 {
            current_withdrawal = withdrawals.first().unwrap().clone();
            if &current_withdrawal.receiver_lock_script != &transfer_lock_script {
                return Err(Error::CotaIdAndTokenIndexHasNotWithdrawn);
            }
            let is_claimed = is_exist_in_claim(
                transfer_lock_hash,
                current_withdrawal.cota_id,
                current_withdrawal.token_index,
                current_withdrawal.out_point,
            );
            if is_claimed {
                return Err(Error::CotaIdAndTokenIndexHasNotWithdrawn);
            }
        }
    }

    let mut action_vec: Vec<u8> = Vec::new();
    action_vec.extend("Transfer the NFT ".as_bytes());
    action_vec.extend(hex_string(&current_withdrawal.cota_id).as_bytes());
    action_vec.extend(hex_string(&current_withdrawal.token_index).as_bytes());
    action_vec.extend(" to ".as_bytes());
    action_vec.extend(hex_string(&current_transfer.to_lock_script).as_bytes());
    let action_bytes = BytesBuilder::default()
        .set(action_vec.iter().map(|v| Byte::from(*v)).collect())
        .build();

    let mut claimed_keys: Vec<ClaimCotaNFTKey> = Vec::new();
    let mut claimed_values: Vec<Byte32> = Vec::new();
    let mut withdrawal_keys: Vec<WithdrawalCotaNFTKeyV1> = Vec::new();
    let mut withdrawal_values: Vec<WithdrawalCotaNFTValueV1> = Vec::new();
    let mut transfer_new_leaves: Vec<(H256, H256)> = Vec::with_capacity(transfers_len * 2);
    let mut transfer_update_leaves: Vec<(H256, H256)> = Vec::with_capacity(2);
    let mut previous_leaves: Vec<(H256, H256)> = Vec::with_capacity(transfers_len * 2);
    for (index, (withdrawal_db, transfer)) in sender_withdrawals
        .into_iter()
        .zip(transfers.clone())
        .enumerate()
    {
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
        let SequentTransfer { to_lock_script, .. } = transfer;

        let (withdrawal_key, key) =
            generate_withdrawal_key_v1(cota_id, token_index, transfer.transfer_out_point);
        let (withdrawal_value, value) =
            generate_withdrawal_value_v1(configure, state, characteristic, &to_lock_script);
        withdrawal_keys.push(withdrawal_key);
        withdrawal_values.push(withdrawal_value);
        previous_leaves.push((key, H256::zero()));
        transfer_new_leaves.push((key, value));
        if index == transfers_len - 1 {
            transfer_update_leaves.push((key, value));
        }

        let (claimed_key, key) = generate_claim_key(cota_id, token_index, out_point);
        let (claimed_value, value) = generate_claim_value(version);
        claimed_keys.push(claimed_key);
        claimed_values.push(claimed_value);
        previous_leaves.push((key, H256::zero()));
        transfer_new_leaves.push((key, value));
        if index == transfers_len - 1 {
            transfer_update_leaves.push((key, value));
        }
    }

    let transfer_smt_root = get_cota_smt_root(&transfer_lock_script).await?;

    let transaction = &StoreTransaction::new(ROCKS_DB.transaction());
    let mut transfer_smt = init_smt(transaction, transfer_lock_hash)?;
    // Add lock to transfer smt
    with_lock(transfer_lock_hash, || {
        generate_history_smt(&mut transfer_smt, transfer_lock_hash, transfer_smt_root)?;
        transfer_smt
            .update_all(transfer_new_leaves.clone())
            .map_err(|e| Error::SMTError(e.to_string()))?;
        transfer_smt.save_root_and_leaves(previous_leaves.clone())?;
        transfer_smt.commit()
    })?;

    let leaf_keys: Vec<H256> = transfer_update_leaves.iter().map(|leave| leave.0).collect();
    let transfer_merkle_proof = transfer_smt
        .merkle_proof(leaf_keys.clone())
        .map_err(|_e| Error::SMTProofError("Transfer".to_string()))?;
    let transfer_merkle_proof_compiled = transfer_merkle_proof
        .compile(leaf_keys)
        .map_err(|_e| Error::SMTProofError("Transfer".to_string()))?;

    let transfer_merkel_proof_vec: Vec<u8> = transfer_merkle_proof_compiled.into();
    let transfer_merkel_proof_bytes = BytesBuilder::default()
        .extend(transfer_merkel_proof_vec.iter().map(|v| Byte::from(*v)))
        .build();

    let withdraw_info = get_withdraw_info(
        current_withdrawal.block_number,
        current_transfer.withdrawal_lock_hash,
    )
    .await?;
    let withdraw_proof = parse_witness_withdraw_proof(
        withdraw_info.witnesses,
        &[(current_transfer.cota_id, current_transfer.token_index)],
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
        .withdrawal_proof(withdraw_proof)
        .leaf_keys(Byte32Vec::default())
        .leaf_values(Byte32Vec::default())
        .raw_tx(withdraw_info.raw_tx)
        .output_index(withdraw_info.output_index)
        .tx_proof(withdraw_info.tx_proof)
        .build();

    let subkey_unlock_entries = generate_subkey_smt(transfer_lock_hash, subkey_opt, &transfer_smt)?;

    Ok((
        *transfer_smt.root(),
        transfer_entries,
        subkey_unlock_entries,
        withdraw_info.block_hash,
    ))
}

fn generate_subkey_smt(
    transfer_lock_hash: [u8; 32],
    subkey_opt: Option<SubKeyUnlock>,
    smt: &CotaSMT,
) -> Result<Option<SubKeyUnlockEntries>, Error> {
    if let Some(subkey) = subkey_opt {
        let subkey =
            get_subkey_by_pubkey_hash(transfer_lock_hash, subkey.pubkey_hash, subkey.alg_index)?
                .ok_or(Error::SubkeyLeafNotFound)?;
        let (_, key) = generate_subkey_key(subkey.ext_data);
        let ext_data = joyid_smt::common::Uint32::from_slice(&subkey.ext_data.to_be_bytes())
            .map_err(|_| Error::Other("Parse uint32 error".to_owned()))?;
        let alg_index = joyid_smt::common::Uint16::from_slice(&subkey.alg_index.to_be_bytes())
            .map_err(|_| Error::Other("Parse uint16 error".to_owned()))?;

        let subkey_merkle_proof = smt
            .merkle_proof(vec![key])
            .map_err(|_| Error::SMTProofError("Subkey unlock".to_owned()))?;
        let subkey_merkle_proof_compiled = subkey_merkle_proof
            .compile(vec![key])
            .map_err(|_| Error::SMTProofError("Subkey unlock".to_owned()))?;

        let merkel_proof_vec: Vec<u8> = subkey_merkle_proof_compiled.into();
        let merkel_proof_bytes = joyid_smt::common::BytesBuilder::default()
            .extend(merkel_proof_vec.iter().map(|v| Byte::from(*v)))
            .build();

        let unlock_entries = SubKeyUnlockEntriesBuilder::default()
            .ext_data(ext_data)
            .alg_index(alg_index)
            .subkey_proof(merkel_proof_bytes)
            .build();
        return Ok(Some(unlock_entries));
    }
    Ok(None)
}
