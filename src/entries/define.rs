use crate::ckb::indexer::get_cota_smt_root;
use crate::entries::helper::{generate_define_key, generate_define_value, with_lock};
use crate::entries::smt::{generate_history_smt, init_smt};
use crate::models::block::get_syncer_tip_block_number;
use crate::request::define::DefineReq;
use crate::smt::transaction::store_transaction::StoreTransaction;
use crate::smt::RootSaver;
use crate::utils::error::Error;
use crate::ROCKS_DB;
use cota_smt::common::*;
use cota_smt::define::{DefineCotaNFTEntries, DefineCotaNFTEntriesBuilder};
use cota_smt::molecule::prelude::*;
use cota_smt::smt::{blake2b_256, H256};
use log::error;

pub async fn generate_define_smt(
    define_req: DefineReq,
) -> Result<(H256, DefineCotaNFTEntries), Error> {
    let mut update_leaves: Vec<(H256, H256)> = Vec::with_capacity(1);
    let mut previous_leaves: Vec<(H256, H256)> = Vec::with_capacity(1);
    let DefineReq {
        cota_id,
        total,
        issued,
        configure,
        ..
    } = define_req;
    let (define_key, key) = generate_define_key(cota_id);
    let block_number = get_syncer_tip_block_number()?;
    let (define_value, value) = generate_define_value(total, issued, configure, block_number);
    update_leaves.push((key, value));
    previous_leaves.push((key, H256::zero()));

    let smt_root = get_cota_smt_root(&define_req.lock_script).await?;
    let transaction = &StoreTransaction::new(ROCKS_DB.transaction());
    let lock_hash = blake2b_256(&define_req.lock_script);
    let mut smt = init_smt(transaction, lock_hash)?;
    // Add lock to smt
    with_lock(lock_hash, || {
        generate_history_smt(&mut smt, lock_hash, smt_root)?;
        smt.update(key, value)
            .map_err(|e| Error::SMTInvalid(e.to_string()))?;
        smt.save_root_and_leaves(previous_leaves.clone())?;
        smt.commit()
    })?;

    let leaf_keys: Vec<H256> = update_leaves.iter().map(|leave| leave.0).collect();
    let define_merkle_proof = smt.merkle_proof(leaf_keys.clone()).map_err(|e| {
        error!("Define SMT proof error: {:?}", e.to_string());
        Error::SMTProofInvalid("Mint".to_string())
    })?;
    let define_merkle_proof_compiled = define_merkle_proof.compile(leaf_keys).map_err(|e| {
        error!("Define SMT proof error: {:?}", e.to_string());
        Error::SMTProofInvalid("Define".to_string())
    })?;

    let merkel_proof_vec: Vec<u8> = define_merkle_proof_compiled.into();
    let merkel_proof_bytes = BytesBuilder::default()
        .extend(merkel_proof_vec.iter().map(|v| Byte::from(*v)))
        .build();

    let mut action_vec: Vec<u8> = Vec::new();
    action_vec.extend("Create a new NFT collection with ".as_bytes());
    let total = u32::from_be_bytes(define_req.total);
    let define_total = if total == 0u32 {
        "unlimited".to_string()
    } else {
        total.to_string()
    };
    action_vec.extend(define_total.as_bytes());
    action_vec.extend(" edition".as_bytes());
    let action_bytes = BytesBuilder::default()
        .set(action_vec.iter().map(|v| Byte::from(*v)).collect())
        .build();

    let define_entries = DefineCotaNFTEntriesBuilder::default()
        .define_keys(
            DefineCotaNFTKeyVecBuilder::default()
                .set(vec![define_key])
                .build(),
        )
        .define_values(
            DefineCotaNFTValueVecBuilder::default()
                .set(vec![define_value])
                .build(),
        )
        .proof(merkel_proof_bytes)
        .action(action_bytes)
        .build();

    Ok((*smt.root(), define_entries))
}
