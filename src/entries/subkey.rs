use crate::ckb::indexer::get_cota_smt_root;
use crate::entries::helper::with_lock;
use crate::entries::smt::init_smt;
use crate::models::extension::subkey::get_subkey_leaf_by_pubkey_hash;
use crate::request::subkey::SubKeyUnlockReq;
use crate::smt::db::db::RocksDB;
use crate::smt::transaction::store_transaction::StoreTransaction;
use crate::utils::error::Error;
use cota_smt::molecule::prelude::*;
use cota_smt::smt::{blake2b_256, H256};
use joyid_smt::common::*;
use joyid_smt::joyid::{SubKeyUnlockEntries, SubKeyUnlockEntriesBuilder};
use log::error;

use super::smt::generate_history_smt;

pub async fn generate_subkey_unlock_smt(
    db: &RocksDB,
    subkey_unlock_req: SubKeyUnlockReq,
) -> Result<SubKeyUnlockEntries, Error> {
    let SubKeyUnlockReq {
        pubkey_hash,
        lock_script,
    } = subkey_unlock_req;
    let lock_hash = blake2b_256(lock_script.clone());

    let leaf = get_subkey_leaf_by_pubkey_hash(pubkey_hash)?.ok_or(Error::SubkeyLeafNotFound)?;
    let key = H256::from(leaf.key);
    let ext_data = Uint32::from_slice(&leaf.key[8..12])
        .map_err(|_| Error::Other("Parse uint32 error".to_owned()))?;
    let alg_index = Uint16::from_slice(&leaf.value[0..2])
        .map_err(|_| Error::Other("Parse uint16 error".to_owned()))?;

    let smt_root = get_cota_smt_root(&lock_script).await?;
    let transaction = &StoreTransaction::new(db.transaction());

    let mut smt = init_smt(transaction, lock_hash)?;
    // Add lock to smt
    with_lock(lock_hash, || {
        generate_history_smt(&mut smt, lock_hash, smt_root)
    })?;

    let subkey_merkle_proof = smt.merkle_proof(vec![key]).map_err(|e| {
        error!("Subkey unlock SMT proof error: {:?}", e.to_string());
        Error::SMTProofError("Subkey unlock".to_string())
    })?;
    let subkey_merkle_proof_compiled = subkey_merkle_proof.compile(vec![key]).map_err(|e| {
        error!("Subkey unlock SMT proof error: {:?}", e.to_string());
        Error::SMTProofError("Subkey unlock".to_string())
    })?;

    let merkel_proof_vec: Vec<u8> = subkey_merkle_proof_compiled.into();
    let merkel_proof_bytes = BytesBuilder::default()
        .extend(merkel_proof_vec.iter().map(|v| Byte::from(*v)))
        .build();

    let unlock_entries = SubKeyUnlockEntriesBuilder::default()
        .ext_data(ext_data)
        .alg_index(alg_index)
        .subkey_proof(merkel_proof_bytes)
        .build();

    Ok(unlock_entries)
}