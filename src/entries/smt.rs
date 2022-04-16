use crate::entries::helper::{
    generate_claim_key, generate_claim_value, generate_cota_index, generate_define_key,
    generate_define_value, generate_hold_key, generate_hold_value, generate_withdrawal_key,
    generate_withdrawal_key_v1, generate_withdrawal_value, generate_withdrawal_value_v1,
};
use crate::models::claim::ClaimDb;
use crate::models::common::get_all_cota_by_lock_hash;
use crate::models::define::DefineDb;
use crate::models::hold::HoldDb;
use crate::models::withdrawal::WithdrawDb;
use crate::smt::db::schema::{
    COLUMN_SMT_BRANCH, COLUMN_SMT_LEAF, COLUMN_SMT_ROOT, COLUMN_SMT_TEMP_LEAVES,
};
use crate::smt::store::smt_store::SMTStore;
use crate::smt::transaction::store_transaction::StoreTransaction;
use crate::smt::CotaSMT;
use crate::utils::error::Error;
use crate::utils::helper::diff_time;
use chrono::prelude::*;
use cota_smt::common::*;
use cota_smt::smt::{blake2b_256, H256};
use log::debug;
use std::collections::HashMap;

pub fn generate_history_smt<'a>(
    transaction: &'a StoreTransaction,
    lock_script: &[u8],
    smt_root_opt: Option<Vec<u8>>,
) -> Result<CotaSMT<'a>, Error> {
    let lock_hash = blake2b_256(lock_script);
    let smt_store = SMTStore::new(
        lock_hash,
        COLUMN_SMT_LEAF,
        COLUMN_SMT_BRANCH,
        COLUMN_SMT_ROOT,
        COLUMN_SMT_TEMP_LEAVES,
        &transaction,
    );
    let root = smt_store
        .get_root()
        .map_err(|_e| Error::SMTError("Get smt root".to_string()))?
        .unwrap_or_default();
    debug!(
        "rocksdb smt root: {:?} of {:?}",
        root,
        hex::encode(lock_hash)
    );
    let mut smt: CotaSMT = CotaSMT::new(root, smt_store);

    if root == H256::zero() {
        return generate_mysql_smt(smt, lock_hash);
    }
    debug!(
        "cota cell smt root: {:?} of {:?}",
        smt_root_opt,
        hex::encode(lock_hash)
    );
    if let Some(smt_root) = smt_root_opt {
        if smt_root.as_slice() == root.as_slice() {
            return Ok(smt);
        }
    }
    smt = reset_smt_temp_leaves(smt)?;
    generate_mysql_smt(smt, lock_hash)
}

fn generate_mysql_smt<'a>(mut smt: CotaSMT<'a>, lock_hash: [u8; 32]) -> Result<CotaSMT<'a>, Error> {
    let start_time = Local::now().timestamp_millis();
    let (defines, holds, withdrawals, claims) = get_all_cota_by_lock_hash(lock_hash)?;
    diff_time(
        start_time,
        "Load all history smt leaves from mysql database",
    );

    let start_time = Local::now().timestamp_millis();
    for define_db in defines {
        let DefineDb {
            cota_id,
            total,
            issued,
            configure,
        } = define_db;
        let (_, key) = generate_define_key(cota_id);
        let (_, value) =
            generate_define_value(total.to_be_bytes(), issued.to_be_bytes(), configure);
        smt.update(key, value).expect("SMT update leave error");
    }
    for hold_db in holds {
        let HoldDb {
            cota_id,
            token_index,
            configure,
            state,
            characteristic,
        } = hold_db;
        let (_, key) = generate_hold_key(cota_id, token_index);
        let (_, value) = generate_hold_value(configure, state, characteristic);
        smt.update(key, value).expect("SMT update leave error");
    }
    let mut withdrawal_map: HashMap<Vec<u8>, u8> = HashMap::new();
    for withdrawal_db in withdrawals {
        let WithdrawDb {
            cota_id,
            token_index,
            configure,
            state,
            characteristic,
            receiver_lock_script,
            out_point,
            version,
        } = withdrawal_db;
        let (key, value) = if version == 0 {
            (
                generate_withdrawal_key(cota_id, token_index).1,
                generate_withdrawal_value(
                    configure,
                    state,
                    characteristic,
                    &receiver_lock_script,
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
                    &receiver_lock_script,
                )
                .1,
            )
        };
        withdrawal_map.insert(generate_cota_index(cota_id, token_index), version);
        smt.update(key, value).expect("SMT update leave error");
    }
    for claim_db in claims {
        let ClaimDb {
            cota_id,
            token_index,
            out_point,
        } = claim_db;
        let version = withdrawal_map
            .get(&*generate_cota_index(cota_id, token_index))
            .cloned()
            .unwrap_or_default();
        let (_, key) = generate_claim_key(cota_id, token_index, out_point);
        let (_, value) = generate_claim_value(version);
        smt.update(key, value).expect("SMT update leave error");
    }
    diff_time(start_time, "Push all history leaves to smt");
    Ok(smt)
}

fn reset_smt_temp_leaves<'a>(mut smt: CotaSMT<'a>) -> Result<CotaSMT<'a>, Error> {
    let leaves_opt = smt.store().get_leaves()?;
    if let Some(leaves) = leaves_opt {
        smt.update_all(leaves)
            .expect("SMT update temp leaves error");
    }
    debug!("Reset temp leaves successfully");
    Ok(smt)
}
