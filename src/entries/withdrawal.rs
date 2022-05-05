use crate::entries::helper::{
    generate_empty_value, generate_hold_key, generate_hold_value, generate_withdrawal_key_v1,
    generate_withdrawal_value_v1,
};
use crate::entries::smt::{generate_history_smt, init_smt};
use crate::entries::SMT_LOCK;
use crate::indexer::index::get_cota_smt_root;
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
use std::sync::Arc;

pub async fn generate_withdrawal_smt(
    db: &RocksDB,
    withdrawal_req: WithdrawalReq,
) -> Result<(H256, WithdrawalCotaNFTV1Entries), Error> {
    let withdrawals = withdrawal_req.withdrawals;
    if withdrawals.is_empty() {
        return Err(Error::RequestParamNotFound("withdrawals".to_string()));
    }
    let cota_id_and_token_index_pairs = Some(
        withdrawals
            .iter()
            .map(|withdrawal| (withdrawal.cota_id, withdrawal.token_index))
            .collect(),
    );
    let db_holds = get_hold_cota_by_lock_hash(
        blake2b_256(&withdrawal_req.lock_script.clone()),
        cota_id_and_token_index_pairs,
    )?
    .0;
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

    let smt_root = get_cota_smt_root(&withdrawal_req.lock_script).await?;
    let transaction = &StoreTransaction::new(db.transaction());
    let lock_hash = blake2b_256(&withdrawal_req.lock_script);
    let mut smt = init_smt(transaction, lock_hash)?;
    // Add lock to smt
    let &(ref lock, ref cond) = &*Arc::clone(&SMT_LOCK);
    {
        let mut set = lock.lock();
        while !set.insert(lock_hash) {
            cond.wait(&mut set);
        }
    }
    let unlock = || {
        let mut set = lock.lock();
        set.remove(&lock_hash);
        cond.notify_all();
    };
    let err_handle = |err| {
        unlock();
        err
    };
    smt = generate_history_smt(smt, lock_hash, smt_root).map_err(err_handle)?;
    smt.update_all(update_leaves.clone()).map_err(|e| {
        unlock();
        Error::SMTError(e.to_string())
    })?;
    smt.save_root_and_leaves(previous_leaves)
        .map_err(err_handle)?;
    smt.commit().map_err(err_handle)?;
    unlock();

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
        action_vec.extend(&withdrawals.first().unwrap().cota_id);
        action_vec.extend(&withdrawals.first().unwrap().token_index);
        action_vec.extend(" to ".as_bytes());
        action_vec.extend(&withdrawals.get(0).unwrap().to_lock_script);
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
