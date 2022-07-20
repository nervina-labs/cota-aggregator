use crate::entries::helper::{
    generate_claim_key, generate_claim_value, generate_cota_index, generate_define_key,
    generate_define_value, generate_hold_key, generate_hold_value, generate_withdrawal_key,
    generate_withdrawal_key_v1, generate_withdrawal_value, generate_withdrawal_value_v1,
};
use crate::models::claim::ClaimDb;
use crate::models::common::{get_all_cota_by_lock_hash, get_all_cota_by_secp256k1_batch_lock};
use crate::models::define::DefineDb;
use crate::models::helper::get_secp256k1_batch_code_hash;
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
use ckb_types::packed::Script;
use cota_smt::common::*;
use cota_smt::smt::{blake2b_256, H256};
use log::debug;
use molecule::prelude::Entity;
use std::collections::HashMap;
use std::convert::TryInto;

pub fn init_smt<'a>(
    transaction: &'a StoreTransaction,
    lock_hash: [u8; 32],
) -> Result<CotaSMT<'a>, Error> {
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
    Ok(CotaSMT::new(root, smt_store))
}

pub fn generate_history_smt<'a>(
    smt: &mut CotaSMT<'a>,
    lock_script: Vec<u8>,
    smt_root_opt: Option<[u8; 32]>,
) -> Result<(), Error> {
    let lock_hash = blake2b_256(&lock_script);
    let master_args = parse_secp256k1_batch_lock_master_args(&lock_script);
    let root = *smt.root();
    if root == H256::zero() {
        return generate_mysql_smt(smt, lock_hash, master_args);
    }
    debug!(
        "cota cell smt root: {:?} of {:?}",
        smt_root_opt,
        hex::encode(lock_hash)
    );
    if let Some(smt_root) = smt_root_opt {
        if smt_root.as_slice() == root.as_slice() {
            debug!("The smt leaves and root in rocksdb are right");
            return Ok(());
        } else {
            reset_smt_temp_leaves(smt)?;
            if smt_root.as_slice() == smt.root().as_slice() {
                debug!("The smt leaves and root in rocksdb are right after reset");
                return Ok(());
            }
        }
    }
    reset_smt_temp_leaves(smt)?;
    generate_mysql_smt(smt, lock_hash, master_args)
}

fn parse_secp256k1_batch_lock_master_args(lock_script: &[u8]) -> Option<[u8; 20]> {
    let script = Script::from_slice(lock_script).unwrap();
    let secp256k1_batch_code_hash = hex::decode(get_secp256k1_batch_code_hash()).unwrap();
    if script.code_hash().as_slice() == &secp256k1_batch_code_hash && script.args().len() >= 20 {
        let args: [u8; 20] = script.args().as_slice()[0..20]
            .try_into()
            .expect("secp256k1_batch lock args should be valid");
        return Some(args);
    }
    None
}

fn generate_mysql_smt<'a>(
    smt: &mut CotaSMT<'a>,
    lock_hash: [u8; 32],
    master_args: Option<[u8; 20]>,
) -> Result<(), Error> {
    let start_time = Local::now().timestamp_millis();
    let (defines, holds, withdrawals, claims) = if let Some(args) = master_args {
        get_all_cota_by_secp256k1_batch_lock(args)?
    } else {
        get_all_cota_by_lock_hash(lock_hash)?
    };
    diff_time(
        start_time,
        "Load all history smt leaves from mysql database",
    );

    let mut leaves: Vec<(H256, H256)> = vec![];
    let start_time = Local::now().timestamp_millis();
    for define_db in defines {
        let DefineDb {
            cota_id,
            total,
            issued,
            configure,
            block_number,
        } = define_db;
        let (_, key) = generate_define_key(cota_id);
        let (_, value) = generate_define_value(
            total.to_be_bytes(),
            issued.to_be_bytes(),
            configure,
            block_number,
        );
        leaves.push((key, value));
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
        leaves.push((key, value));
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
            ..
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
        leaves.push((key, value));
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
        leaves.push((key, value));
    }
    if !leaves.is_empty() {
        smt.update_all(leaves).expect("SMT update leave error");
    }
    diff_time(start_time, "Push all history leaves to smt");
    debug!("The smt root of all leaves from mysql: {:?}", smt.root());
    Ok(())
}

fn reset_smt_temp_leaves<'a>(smt: &mut CotaSMT<'a>) -> Result<(), Error> {
    let leaves_opt = smt.store().get_leaves()?;
    if let Some(leaves) = leaves_opt {
        smt.update_all(leaves)
            .expect("SMT update temp leaves error");
    }
    debug!("Reset temp leaves successfully");
    Ok(())
}
