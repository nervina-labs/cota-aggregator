use crate::entries::helper::{generate_define_key, generate_define_value};
use crate::entries::smt::generate_history_smt;
use crate::request::define::DefineReq;
use crate::smt::db::db::RocksDB;
use crate::smt::transaction::store_transaction::StoreTransaction;
use crate::smt::RootSaver;
use crate::utils::error::Error;
use cota_smt::common::*;
use cota_smt::define::{DefineCotaNFTEntries, DefineCotaNFTEntriesBuilder};
use cota_smt::molecule::prelude::*;
use cota_smt::smt::H256;
use log::error;

pub async fn generate_define_smt(
    db: &RocksDB,
    define_req: DefineReq,
) -> Result<(H256, DefineCotaNFTEntries), Error> {
    let transaction = &StoreTransaction::new(db.transaction());
    let mut smt = generate_history_smt(transaction, define_req.lock_script.as_slice()).await?;

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
    let (define_value, value) = generate_define_value(total, issued, configure);

    smt.update(key, value)
        .expect("define SMT update leave error");
    update_leaves.push((key, value));
    previous_leaves.push((key, H256::zero()));

    smt.save_root_and_leaves(previous_leaves)?;
    let define_merkle_proof = smt
        .merkle_proof(update_leaves.iter().map(|leave| leave.0).collect())
        .map_err(|e| {
            error!("Define SMT proof error: {:?}", e.to_string());
            Error::SMTProofError("Mint".to_string())
        })?;
    let define_merkle_proof_compiled =
        define_merkle_proof
            .compile(update_leaves.clone())
            .map_err(|e| {
                error!("Define SMT proof error: {:?}", e.to_string());
                Error::SMTProofError("Define".to_string())
            })?;

    let merkel_proof_vec: Vec<u8> = define_merkle_proof_compiled.into();
    let merkel_proof_bytes = BytesBuilder::default()
        .extend(merkel_proof_vec.iter().map(|v| Byte::from(*v)))
        .build();

    let mut action_vec: Vec<u8> = Vec::new();
    action_vec.extend("Create a new NFT collection with ".as_bytes());
    let define_total = if u32::from_be_bytes(define_req.total) == 0u32 {
        "unlimited".as_bytes()
    } else {
        &define_req.total
    };
    action_vec.extend(define_total);
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
