use crate::ckb::indexer::get_cota_smt_root;
use crate::entries::helper::{
    generate_empty_value, generate_hold_key, generate_hold_value, generate_withdrawal_key_v1,
    generate_withdrawal_value_v1, with_lock,
};
use crate::entries::smt::{generate_history_smt, init_smt};
use crate::models::hold::get_hold_cota_by_lock_hash;
use crate::request::withdrawal::WithdrawalReq;
use crate::smt::db::db::RocksDB;
use crate::smt::transaction::store_transaction::StoreTransaction;
use crate::smt::RootSaver;
use crate::utils::error::Error;
use cota_smt::common::*;
use cota_smt::molecule::prelude::*;
use cota_smt::smt::{blake2b_256, H256};
use cota_smt::transfer::{WithdrawalCotaNFTV1Entries, WithdrawalCotaNFTV1EntriesBuilder};
use log::error;
use molecule::hex_string;

pub async fn generate_withdrawal_smt(
    db: &RocksDB,
    withdrawal_req: WithdrawalReq,
) -> Result<(H256, WithdrawalCotaNFTV1Entries), Error> {
    let withdrawals = withdrawal_req.withdrawals;
    let withdrawal_lock_script = withdrawal_req.lock_script;
    if withdrawals.is_empty() {
        return Err(Error::RequestParamNotFound("withdrawals".to_string()));
    }
    let cota_id_index_pairs: Vec<([u8; 20], [u8; 4])> = withdrawals
        .iter()
        .map(|withdrawal| (withdrawal.cota_id, withdrawal.token_index))
        .collect();
    let withdrawal_lock_hash = blake2b_256(&withdrawal_lock_script);
    let db_holds = get_hold_cota_by_lock_hash(withdrawal_lock_hash, &cota_id_index_pairs)?.0;
    if db_holds.is_empty() || db_holds.len() != withdrawals.len() {
        return Err(Error::CotaIdAndTokenIndexHasNotHeld);
    }
    let mut hold_keys: Vec<CotaNFTId> = Vec::new();
    let mut hold_values: Vec<CotaNFTInfo> = Vec::new();
    let mut withdrawal_keys: Vec<WithdrawalCotaNFTKeyV1> = Vec::new();
    let mut withdrawal_values: Vec<WithdrawalCotaNFTValueV1> = Vec::new();
    let mut update_leaves: Vec<(H256, H256)> = Vec::with_capacity(withdrawals.len() * 2);
    let mut previous_leaves: Vec<(H256, H256)> = Vec::with_capacity(withdrawals.len() * 2);
    for (hold_db, withdrawal) in db_holds.iter().zip(withdrawals.iter()) {
        let (hold_key, key) = generate_hold_key(hold_db.cota_id, hold_db.token_index);
        let (hold_value, old_value) =
            generate_hold_value(hold_db.configure, hold_db.state, hold_db.characteristic);
        let (_, value) = generate_empty_value();
        hold_keys.push(hold_key);
        hold_values.push(hold_value);
        update_leaves.push((key, value));
        previous_leaves.push((key, old_value));

        let (withdrawal_key, key) = generate_withdrawal_key_v1(
            hold_db.cota_id,
            hold_db.token_index,
            withdrawal_req.out_point,
        );
        withdrawal_keys.push(withdrawal_key);

        let (withdrawal_value, value) = generate_withdrawal_value_v1(
            hold_db.configure,
            hold_db.state,
            hold_db.characteristic,
            &withdrawal.to_lock_script,
        );
        withdrawal_values.push(withdrawal_value);
        update_leaves.push((key, value));
        previous_leaves.push((key, H256::zero()));
    }

    let smt_root = get_cota_smt_root(&withdrawal_lock_script).await?;
    let transaction = &StoreTransaction::new(db.transaction());
    let mut smt = init_smt(transaction, withdrawal_lock_hash)?;
    // Add lock to smt
    with_lock(withdrawal_lock_hash, || {
        generate_history_smt(&mut smt, withdrawal_lock_script.clone(), smt_root)?;
        smt.update_all(update_leaves.clone())
            .map_err(|e| Error::SMTError(e.to_string()))?;
        smt.save_root_and_leaves(previous_leaves.clone())?;
        smt.commit()
    })?;

    let withdrawal_merkle_proof = smt
        .merkle_proof(update_leaves.iter().map(|leave| leave.0).collect())
        .map_err(|e| {
            error!("Withdraw SMT proof error: {:?}", e.to_string());
            Error::SMTProofError("Withdraw".to_string())
        })?;
    let withdrawal_merkle_proof_compiled = withdrawal_merkle_proof
        .compile(update_leaves.clone())
        .map_err(|e| {
        error!("Withdraw SMT proof error: {:?}", e.to_string());
        Error::SMTProofError("Withdraw".to_string())
    })?;

    let merkel_proof_vec: Vec<u8> = withdrawal_merkle_proof_compiled.into();
    let merkel_proof_bytes = BytesBuilder::default()
        .extend(merkel_proof_vec.iter().map(|v| Byte::from(*v)))
        .build();

    let mut action_vec: Vec<u8> = Vec::new();
    if withdrawals.len() == 1 {
        action_vec.extend("Transfer the NFT ".as_bytes());
        action_vec.extend(hex_string(&withdrawals.first().unwrap().cota_id).as_bytes());
        action_vec.extend(hex_string(&withdrawals.first().unwrap().token_index).as_bytes());
        action_vec.extend(" to ".as_bytes());
        action_vec.extend(hex_string(&withdrawals.first().unwrap().to_lock_script).as_bytes());
    }
    let action_bytes = BytesBuilder::default()
        .set(action_vec.iter().map(|v| Byte::from(*v)).collect())
        .build();

    let withdrawal_entries = WithdrawalCotaNFTV1EntriesBuilder::default()
        .hold_keys(HoldCotaNFTKeyVecBuilder::default().set(hold_keys).build())
        .hold_values(
            HoldCotaNFTValueVecBuilder::default()
                .set(hold_values)
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
        .proof(merkel_proof_bytes)
        .action(action_bytes)
        .build();

    Ok((*smt.root(), withdrawal_entries))
}
