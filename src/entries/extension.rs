use super::helper::{generate_extension_key, generate_extension_value};
use crate::ckb::indexer::get_cota_smt_root;
use crate::entries::helper::with_lock;
use crate::entries::smt::{generate_history_smt, init_smt};
use crate::models::extension::leaves::get_extension_leaf_by_lock_hash;
use crate::request::extension::ExtensionReq;
use crate::smt::db::db::RocksDB;
use crate::smt::transaction::store_transaction::StoreTransaction;
use crate::smt::RootSaver;
use crate::utils::error::Error;
use cota_smt::common::*;
use cota_smt::extension::{
    ExtensionEntries, ExtensionEntriesBuilder, ExtensionLeavesBuilder, ExtensionVecBuilder,
};
use cota_smt::molecule::prelude::*;
use cota_smt::smt::{blake2b_256, H256};
use log::error;

pub async fn generate_extension_smt(
    db: &RocksDB,
    extension_req: ExtensionReq,
) -> Result<(H256, ExtensionEntries), Error> {
    let mut update_leaves: Vec<(H256, H256)> = Vec::with_capacity(1);
    let mut previous_leaves: Vec<(H256, H256)> = Vec::with_capacity(1);
    let ExtensionReq { lock_script } = extension_req;
    let lock_hash = blake2b_256(&lock_script);

    let (ext_key, key) = generate_extension_key();
    let (ext_value, value) = generate_extension_value();

    let leaf_opt = get_extension_leaf_by_lock_hash(lock_hash, key)?;
    let old_values = match leaf_opt {
        Some(leaf) => vec![Byte32::from_slice(&leaf.value).unwrap()],
        None => vec![],
    };

    update_leaves.push((key, value));
    previous_leaves.push((key, H256::zero()));

    let smt_root = get_cota_smt_root(&lock_script).await?;
    let transaction = &StoreTransaction::new(db.transaction());

    let mut smt = init_smt(transaction, lock_hash)?;
    // Add lock to smt
    with_lock(lock_hash, || {
        generate_history_smt(&mut smt, lock_hash, smt_root)?;
        smt.update(key, value)
            .map_err(|e| Error::SMTError(e.to_string()))?;
        smt.save_root_and_leaves(previous_leaves.clone())?;
        smt.commit()
    })?;

    let leaf_keys: Vec<H256> = update_leaves.iter().map(|leave| leave.0).collect();
    let extension_merkle_proof = smt.merkle_proof(leaf_keys.clone()).map_err(|e| {
        error!("Extension SMT proof error: {:?}", e.to_string());
        Error::SMTProofError("Extension".to_string())
    })?;
    let extension_merkle_proof_compiled =
        extension_merkle_proof.compile(leaf_keys).map_err(|e| {
            error!("Extension SMT proof error: {:?}", e.to_string());
            Error::SMTProofError("Extension".to_string())
        })?;

    let merkel_proof_vec: Vec<u8> = extension_merkle_proof_compiled.into();
    let merkel_proof_bytes = BytesBuilder::default()
        .extend(merkel_proof_vec.iter().map(|v| Byte::from(*v)))
        .build();

    let extension_entries = ExtensionEntriesBuilder::default()
        .leaves(
            ExtensionLeavesBuilder::default()
                .keys(ExtensionVecBuilder::default().set(vec![ext_key]).build())
                .values(ExtensionVecBuilder::default().set(vec![ext_value]).build())
                .old_values(ExtensionVecBuilder::default().set(old_values).build())
                .proof(merkel_proof_bytes)
                .build(),
        )
        .sub_type(Byte6::from_slice("subkey".as_bytes()).unwrap())
        .raw_data(Bytes::default())
        .build();

    Ok((*smt.root(), extension_entries))
}
