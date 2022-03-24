use crate::entries::helper::{generate_define_key, generate_define_value, save_smt_root_and_keys};
use crate::models::define::{get_define_cota_by_lock_hash, DefineDb};
use crate::request::define::DefineReq;
use crate::smt::db::cota_db::CotaRocksDB;
use crate::smt::smt::generate_history_smt;
use crate::utils::error::Error;
use cota_smt::common::*;
use cota_smt::define::DefineCotaNFTEntriesBuilder;
use cota_smt::molecule::prelude::*;
use cota_smt::smt::{blake2b_256, H256};
use log::error;

pub async fn generate_define_smt(define_req: DefineReq) -> Result<(String, String), Error> {
    let db = CotaRocksDB::default();
    let mut smt = generate_history_smt(&db, define_req.lock_script.clone()).await?;
    let db_defines = get_define_cota_by_lock_hash(blake2b_256(&define_req.lock_script.clone()))?.0;
    if !db_defines.is_empty() {
        for DefineDb {
            cota_id,
            total,
            issued,
            configure,
        } in db_defines
        {
            let (_, key) = generate_define_key(cota_id);
            let (_, value) =
                generate_define_value(total.to_be_bytes(), issued.to_be_bytes(), configure);
            smt.update(key, value)
                .expect("define SMT update leave error");
        }
    }

    let mut update_leaves: Vec<(H256, H256)> = Vec::with_capacity(1);
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

    let root_hash = smt.root().clone();
    let mut root_hash_bytes = [0u8; 32];
    root_hash_bytes.copy_from_slice(root_hash.as_slice());
    let root_hash_hex = hex::encode(root_hash_bytes);

    let update_keys: Vec<H256> = update_leaves.iter().map(|leave| leave.0).collect();
    save_smt_root_and_keys(&smt, "Define", Some(update_keys.clone()));
    let define_merkle_proof = smt.merkle_proof(update_keys).map_err(|e| {
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

    let define_entry = hex::encode(define_entries.as_slice());

    Ok((root_hash_hex, define_entry))
}
