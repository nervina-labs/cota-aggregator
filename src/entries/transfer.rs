use crate::entries::helper::{
    generate_claim_key, generate_claim_value, generate_withdrawal_key, generate_withdrawal_key_v1,
    generate_withdrawal_value, generate_withdrawal_value_v1, lock_err_handle, smt_lock, smt_unlock,
};
use crate::entries::smt::{generate_history_smt, init_smt};
use crate::indexer::index::get_cota_smt_root;
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
use cota_smt::transfer::{TransferCotaNFTV1Entries, TransferCotaNFTV1EntriesBuilder};
use log::error;

pub async fn generate_transfer_smt(
    db: &RocksDB,
    transfer_req: TransferReq,
) -> Result<(H256, TransferCotaNFTV1Entries), Error> {
    let transfers = transfer_req.transfers;
    let transfers_len = transfers.len();
    if transfers_len == 0 {
        return Err(Error::RequestParamNotFound("transfers".to_string()));
    }
    let cota_id_and_token_index_pairs = Some(
        transfers
            .iter()
            .map(|transfer| (transfer.cota_id, transfer.token_index))
            .collect(),
    );
    let withdraw_lock_hash = blake2b_256(&transfer_req.withdrawal_lock_script);
    let sender_withdrawals =
        get_withdrawal_cota_by_lock_hash(withdraw_lock_hash, cota_id_and_token_index_pairs)?.0;
    if sender_withdrawals.is_empty() || sender_withdrawals.len() != transfers_len {
        return Err(Error::CotaIdAndTokenIndexHasNotWithdrawn);
    }

    let mut action_vec: Vec<u8> = Vec::new();
    if transfers_len == 1 {
        action_vec.extend("Transfer the NFT ".as_bytes());
        action_vec.extend(&sender_withdrawals.first().unwrap().cota_id);
        action_vec.extend(&sender_withdrawals.first().unwrap().token_index);
        action_vec.extend(" to ".as_bytes());
        action_vec.extend(&transfers.first().unwrap().to_lock_script);
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
    let mut withdrawal_update_leaves: Vec<(H256, H256)> = Vec::with_capacity(transfers_len);
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

        let (key, value) = if version == 0 {
            (
                generate_withdrawal_key(cota_id, token_index).1,
                generate_withdrawal_value(
                    configure,
                    state,
                    characteristic,
                    &transfer_req.lock_script,
                    out_point,
                )
                .1,
            )
        } else {
            (
                generate_withdrawal_key_v1(cota_id, token_index, out_point).1,
                generate_withdrawal_value_v1(
                    configure,
                    state,
                    characteristic,
                    &transfer_req.lock_script,
                )
                .1,
            )
        };
        withdrawal_update_leaves.push((key, value));

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
    let withdrawal_smt_root = get_cota_smt_root(&transfer_req.withdrawal_lock_script).await?;

    let transaction = &StoreTransaction::new(db.transaction());
    let transfer_lock_hash = blake2b_256(&transfer_req.lock_script);
    let mut transfer_smt = init_smt(transaction, transfer_lock_hash)?;
    // Add lock to transfer smt
    smt_lock(transfer_lock_hash);
    let err_handle = |err| lock_err_handle(&transfer_lock_hash, err);
    transfer_smt = generate_history_smt(transfer_smt, transfer_lock_hash, transfer_smt_root)
        .map_err(err_handle)?;
    transfer_smt
        .update_all(transfer_update_leaves.clone())
        .map_err(|e| {
            smt_unlock(&transfer_lock_hash);
            Error::SMTError(e.to_string())
        })?;
    transfer_smt
        .save_root_and_leaves(previous_leaves)
        .map_err(err_handle)?;
    transfer_smt.commit().map_err(err_handle)?;
    smt_unlock(&transfer_lock_hash);

    let transaction = &StoreTransaction::new(db.transaction());
    let mut withdrawal_smt = init_smt(transaction, withdraw_lock_hash)?;
    // Add lock to withdraw smt
    smt_lock(withdraw_lock_hash);
    let err_handle = |err| lock_err_handle(&withdraw_lock_hash, err);
    withdrawal_smt = generate_history_smt(withdrawal_smt, withdraw_lock_hash, withdrawal_smt_root)
        .map_err(err_handle)?;
    withdrawal_smt
        .save_root_and_leaves(vec![])
        .map_err(err_handle)?;
    withdrawal_smt.commit().map_err(err_handle)?;
    smt_unlock(&withdraw_lock_hash);

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

    let start_time = Local::now().timestamp_millis();
    let withdrawal_merkle_proof = withdrawal_smt
        .merkle_proof(
            withdrawal_update_leaves
                .iter()
                .map(|leave| leave.0)
                .collect(),
        )
        .map_err(|e| {
            error!("Transfer SMT proof error: {:?}", e.to_string());
            Error::SMTProofError("Transfer".to_string())
        })?;
    let withdrawal_merkle_proof_compiled = withdrawal_merkle_proof
        .compile(withdrawal_update_leaves.clone())
        .map_err(|e| {
            error!("Transfer SMT proof error: {:?}", e.to_string());
            Error::SMTProofError("Transfer".to_string())
        })?;
    diff_time(start_time, "Generate withdraw smt proof");

    let withdrawal_merkel_proof_vec: Vec<u8> = withdrawal_merkle_proof_compiled.into();
    let withdrawal_merkel_proof_bytes = BytesBuilder::default()
        .extend(withdrawal_merkel_proof_vec.iter().map(|v| Byte::from(*v)))
        .build();

    let transfer_entries = TransferCotaNFTV1EntriesBuilder::default()
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
        .withdrawal_proof(withdrawal_merkel_proof_bytes)
        .action(action_bytes)
        .build();

    Ok((*transfer_smt.root(), transfer_entries))
}
