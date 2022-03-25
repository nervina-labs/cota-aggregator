use crate::entries::helper::{
    generate_claim_key, generate_claim_value, generate_hold_key, generate_hold_value,
    generate_withdrawal_key, generate_withdrawal_key_v1, generate_withdrawal_value,
    generate_withdrawal_value_v1,
};
use crate::entries::smt::generate_history_smt;
use crate::models::withdrawal::{get_withdrawal_cota_by_lock_hash, WithdrawDb};
use crate::request::claim::ClaimUpdateReq;
use crate::smt::db::cota_db::CotaRocksDB;
use crate::utils::error::Error;
use cota_smt::common::*;
use cota_smt::molecule::prelude::*;
use cota_smt::smt::{blake2b_256, H256};
use cota_smt::transfer_update::ClaimUpdateCotaNFTEntriesBuilder;
use log::error;

pub async fn generate_claim_update_smt(
    claim_update_req: ClaimUpdateReq,
) -> Result<(String, String), Error> {
    let nfts = claim_update_req.nfts.clone();
    let nfts_len = nfts.len();
    if nfts_len == 0 {
        return Err(Error::RequestParamNotFound("nfts".to_string()));
    }
    let cota_id_and_token_index_pairs = Some(
        nfts.iter()
            .map(|nft| (nft.cota_id, nft.token_index))
            .collect(),
    );
    let sender_withdrawals = get_withdrawal_cota_by_lock_hash(
        blake2b_256(claim_update_req.withdrawal_lock_script.clone()),
        cota_id_and_token_index_pairs,
    )?
    .0;
    if sender_withdrawals.is_empty() || sender_withdrawals.len() != nfts_len {
        return Err(Error::CotaIdAndTokenIndexHasNotWithdrawn);
    }

    let mut hold_keys: Vec<CotaNFTId> = Vec::new();
    let mut hold_values: Vec<CotaNFTInfo> = Vec::new();
    let db = CotaRocksDB::default();
    let withdrawal_smt =
        generate_history_smt(&db, claim_update_req.withdrawal_lock_script.clone()).await?;
    let mut withdrawal_update_leaves: Vec<(H256, H256)> = Vec::with_capacity(nfts_len);

    let mut claim_keys: Vec<ClaimCotaNFTKey> = Vec::new();
    let mut key_vec: Vec<(H256, u8)> = Vec::new();
    let mut claim_values: Vec<Byte32> = Vec::new();
    let mut claim_infos: Vec<ClaimCotaNFTInfo> = Vec::new();
    let mut claim_smt = generate_history_smt(&db, claim_update_req.lock_script.clone()).await?;
    let mut claim_update_leaves: Vec<(H256, H256)> = Vec::with_capacity(nfts_len * 2);
    for (index, withdrawal) in sender_withdrawals.into_iter().enumerate() {
        let WithdrawDb {
            cota_id,
            token_index,
            characteristic,
            state,
            configure,
            out_point,
            version,
            ..
        } = withdrawal;
        let (key, value) = if version == 0 {
            (
                generate_withdrawal_key(cota_id, token_index).1,
                generate_withdrawal_value(
                    configure,
                    state,
                    characteristic,
                    claim_update_req.lock_script.clone(),
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
                    claim_update_req.lock_script.clone(),
                )
                .1,
            )
        };
        withdrawal_update_leaves.push((key, value));
        let nft_info = CotaNFTInfoBuilder::default()
            .characteristic(Characteristic::from_slice(&characteristic).unwrap())
            .configure(Byte::from(configure))
            .state(Byte::from(state))
            .build();
        let claim_info = ClaimCotaNFTInfoBuilder::default()
            .nft_info(nft_info)
            .version(Byte::from(version))
            .build();
        claim_infos.push(claim_info);

        let nft = nfts.get(index).ok_or(Error::Other(
            "Get nft from claim_update request error".to_string(),
        ))?;
        let (hold_key, key) = generate_hold_key(cota_id, token_index);
        let (hold_value, value) = generate_hold_value(configure, nft.state, nft.characteristic);
        hold_keys.push(hold_key);
        hold_values.push(hold_value);
        claim_smt
            .update(key, value)
            .expect("claim SMT update leave error");
        claim_update_leaves.push((key, value));

        let (claim_key, key) = generate_claim_key(cota_id, token_index, out_point);
        claim_keys.push(claim_key);
        key_vec.push((key, version));
    }
    let withdraw_merkle_proof = withdrawal_smt
        .merkle_proof(
            withdrawal_update_leaves
                .iter()
                .map(|leave| leave.0)
                .collect(),
        )
        .map_err(|e| {
            error!("Withdraw SMT proof error: {:?}", e.to_string());
            Error::SMTProofError("Withdraw".to_string())
        })?;
    let withdraw_merkle_proof_compiled = withdraw_merkle_proof
        .compile(withdrawal_update_leaves.clone())
        .map_err(|e| {
            error!("Withdraw SMT proof error: {:?}", e.to_string());
            Error::SMTProofError("Withdraw".to_string())
        })?;

    let merkel_proof_vec: Vec<u8> = withdraw_merkle_proof_compiled.into();
    let withdrawal_proof = BytesBuilder::default()
        .extend(merkel_proof_vec.iter().map(|v| Byte::from(*v)))
        .build();

    for (key, version) in key_vec {
        let (claim_value, value) = generate_claim_value(version);
        claim_values.push(claim_value);
        claim_smt
            .update(key, value)
            .expect("claim SMT update leave error");
        claim_update_leaves.push((key, value))
    }
    let claim_update_root_hash = claim_smt.root().clone();
    let mut root_hash_bytes = [0u8; 32];
    root_hash_bytes.copy_from_slice(claim_update_root_hash.as_slice());
    let claim_update_root_hash_hex = hex::encode(root_hash_bytes);

    let claim_update_merkle_proof = claim_smt
        .merkle_proof(claim_update_leaves.iter().map(|leave| leave.0).collect())
        .map_err(|e| {
            error!("Claim update SMT proof error: {:?}", e.to_string());
            Error::SMTProofError("ClaimUpdate".to_string())
        })?;
    let claim_update_merkle_proof_compiled = claim_update_merkle_proof
        .compile(claim_update_leaves.clone())
        .map_err(|e| {
            error!("Claim update SMT proof error: {:?}", e.to_string());
            Error::SMTProofError("ClaimUpdate".to_string())
        })?;

    let merkel_proof_vec: Vec<u8> = claim_update_merkle_proof_compiled.into();
    let claim_proof = BytesBuilder::default()
        .extend(merkel_proof_vec.iter().map(|v| Byte::from(*v)))
        .build();

    let mut action_vec: Vec<u8> = Vec::new();
    action_vec.extend("Claim ".as_bytes());
    action_vec.extend(&[0u8, 0, 0, nfts_len as u8]);
    action_vec.extend(" NFTs and update NFTs information".as_bytes());
    let action_bytes = BytesBuilder::default()
        .set(action_vec.iter().map(|v| Byte::from(*v)).collect())
        .build();

    let claim_update_entries = ClaimUpdateCotaNFTEntriesBuilder::default()
        .hold_keys(HoldCotaNFTKeyVecBuilder::default().set(hold_keys).build())
        .hold_values(
            HoldCotaNFTValueVecBuilder::default()
                .set(hold_values)
                .build(),
        )
        .claim_keys(ClaimCotaNFTKeyVecBuilder::default().set(claim_keys).build())
        .claim_infos(
            ClaimCotaNFTInfoVecBuilder::default()
                .set(claim_infos)
                .build(),
        )
        .proof(claim_proof)
        .withdrawal_proof(withdrawal_proof)
        .action(action_bytes)
        .build();

    let claim_update_entry = hex::encode(claim_update_entries.as_slice());

    Ok((claim_update_root_hash_hex, claim_update_entry))
}
