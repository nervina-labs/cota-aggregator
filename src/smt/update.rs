use crate::models::hold::get_hold_cota_by_lock_hash;
use crate::request::update::UpdateReq;
use crate::smt::common::{generate_history_smt, generate_hold_key, generate_hold_value};
use crate::utils::error::Error;
use cota_smt::common::*;
use cota_smt::molecule::prelude::*;
use cota_smt::smt::H256;
use cota_smt::update::UpdateCotaNFTEntriesBuilder;
use log::error;

pub fn generate_update_smt(update_req: UpdateReq) -> Result<(String, String), Error> {
    let mut smt = generate_history_smt(update_req.lock_hash)?;
    let nfts = update_req.nfts;
    if nfts.is_empty() {
        return Err(Error::RequestParamNotFound("nfts".to_string()));
    }
    let cota_id_and_token_index_pairs = Some(
        nfts.iter()
            .map(|nft| (nft.cota_id, nft.token_index))
            .collect(),
    );
    let db_holds =
        get_hold_cota_by_lock_hash(update_req.lock_hash, cota_id_and_token_index_pairs)?.0;
    if db_holds.is_empty() || db_holds.len() != nfts.len() {
        return Err(Error::CotaIdAndTokenIndexHasNotHeld);
    }
    let mut hold_keys: Vec<CotaNFTId> = Vec::new();
    let mut hold_old_values: Vec<CotaNFTInfo> = Vec::new();
    let mut hold_values: Vec<CotaNFTInfo> = Vec::new();
    let mut update_leaves: Vec<(H256, H256)> = Vec::with_capacity(nfts.len());
    for (hold_db, nft) in db_holds.iter().zip(nfts.iter()) {
        let (hold_key, key) = generate_hold_key(hold_db.cota_id, hold_db.token_index);
        hold_keys.push(hold_key);
        let (hold_old_value, _) =
            generate_hold_value(hold_db.configure, hold_db.state, hold_db.characteristic);
        hold_old_values.push(hold_old_value);
        let (hold_value, value) =
            generate_hold_value(hold_db.configure, nft.state, nft.characteristic);
        hold_values.push(hold_value);
        update_leaves.push((key, value));
        smt.update(key, value).expect("hold SMT update leave error");
    }

    let root_hash = smt.root().clone();
    let mut root_hash_bytes = [0u8; 32];
    root_hash_bytes.copy_from_slice(root_hash.as_slice());
    let root_hash_hex = hex::encode(root_hash_bytes);

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

    let update_entry = hex::encode(update_entries.as_slice());

    Ok((root_hash_hex, update_entry))
}
