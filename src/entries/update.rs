use crate::entries::helper::{generate_hold_key, generate_hold_value};
use crate::entries::smt::generate_history_smt;
use crate::models::hold::get_hold_cota_by_lock_hash;
use crate::request::update::UpdateReq;
use crate::smt::db::cota_db::CotaRocksDB;
use crate::smt::RootSaver;
use crate::utils::error::Error;
use cota_smt::common::*;
use cota_smt::molecule::prelude::*;
use cota_smt::smt::{blake2b_256, H256};
use cota_smt::update::{UpdateCotaNFTEntries, UpdateCotaNFTEntriesBuilder};
use log::error;

pub async fn generate_update_smt(
    update_req: UpdateReq,
) -> Result<(H256, UpdateCotaNFTEntries), Error> {
    let db = CotaRocksDB::default();
    let mut smt = generate_history_smt(&db, update_req.lock_script.as_slice()).await?;
    let nfts = update_req.nfts;
    if nfts.is_empty() {
        return Err(Error::RequestParamNotFound("nfts".to_string()));
    }
    let cota_id_and_token_index_pairs = Some(
        nfts.iter()
            .map(|nft| (nft.cota_id, nft.token_index))
            .collect(),
    );
    let db_holds = get_hold_cota_by_lock_hash(
        blake2b_256(&update_req.lock_script.as_slice()),
        cota_id_and_token_index_pairs,
    )?
    .0;
    if db_holds.is_empty() || db_holds.len() != nfts.len() {
        return Err(Error::CotaIdAndTokenIndexHasNotHeld);
    }
    let mut hold_keys: Vec<CotaNFTId> = Vec::new();
    let mut hold_old_values: Vec<CotaNFTInfo> = Vec::new();
    let mut hold_values: Vec<CotaNFTInfo> = Vec::new();
    let mut update_leaves: Vec<(H256, H256)> = Vec::with_capacity(nfts.len());
    let mut previous_leaves: Vec<(H256, H256)> = Vec::with_capacity(nfts.len());
    for (hold_db, nft) in db_holds.iter().zip(nfts.iter()) {
        let (hold_key, key) = generate_hold_key(hold_db.cota_id, hold_db.token_index);
        let (hold_old_value, old_value) =
            generate_hold_value(hold_db.configure, hold_db.state, hold_db.characteristic);
        let (hold_value, value) =
            generate_hold_value(hold_db.configure, nft.state, nft.characteristic);
        hold_keys.push(hold_key);
        hold_old_values.push(hold_old_value);
        hold_values.push(hold_value);
        update_leaves.push((key, value));
        previous_leaves.push((key, old_value));
        smt.update(key, value).expect("hold SMT update leave error");
    }

    smt.save_root_and_leaves(previous_leaves)?;
    let update_merkle_proof = smt
        .merkle_proof(update_leaves.iter().map(|leave| leave.0).collect())
        .map_err(|e| {
            error!("Update SMT proof error: {:?}", e.to_string());
            Error::SMTProofError("Update".to_string())
        })?;
    let update_merkle_proof_compiled =
        update_merkle_proof
            .compile(update_leaves.clone())
            .map_err(|e| {
                error!("Update SMT proof error: {:?}", e.to_string());
                Error::SMTProofError("Update".to_string())
            })?;

    let merkel_proof_vec: Vec<u8> = update_merkle_proof_compiled.into();
    let merkel_proof_bytes = BytesBuilder::default()
        .extend(merkel_proof_vec.iter().map(|v| Byte::from(*v)))
        .build();

    let mut action_vec: Vec<u8> = Vec::new();
    action_vec.extend("Update NFT information".as_bytes());
    let action_bytes = BytesBuilder::default()
        .set(action_vec.iter().map(|v| Byte::from(*v)).collect())
        .build();

    let update_entries = UpdateCotaNFTEntriesBuilder::default()
        .hold_keys(HoldCotaNFTKeyVecBuilder::default().set(hold_keys).build())
        .hold_old_values(
            HoldCotaNFTValueVecBuilder::default()
                .set(hold_old_values)
                .build(),
        )
        .hold_new_values(
            HoldCotaNFTValueVecBuilder::default()
                .set(hold_values)
                .build(),
        )
        .proof(merkel_proof_bytes)
        .action(action_bytes)
        .build();

    Ok((*smt.root(), update_entries))
}