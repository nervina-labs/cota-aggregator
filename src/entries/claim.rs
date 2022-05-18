use crate::ckb::indexer::get_cota_smt_root;
use crate::ckb::rpc::get_withdraw_info;
use crate::entries::helper::{
    generate_claim_key, generate_claim_value, generate_hold_key, generate_hold_value, with_lock,
};
use crate::entries::smt::{generate_history_smt, init_smt};
use crate::entries::witness::parse_withdraw_witness;
use crate::models::withdrawal::{get_withdrawal_cota_by_lock_hash, WithdrawDb};
use crate::request::claim::ClaimReq;
use crate::smt::db::db::RocksDB;
use crate::smt::transaction::store_transaction::StoreTransaction;
use crate::smt::RootSaver;
use crate::utils::error::Error;
use cota_smt::common::*;
use cota_smt::molecule::prelude::*;
use cota_smt::smt::{blake2b_256, H256};
use cota_smt::transfer::{ClaimCotaNFTV2Entries, ClaimCotaNFTV2EntriesBuilder};
use log::error;

pub async fn generate_claim_smt(
    db: &RocksDB,
    claim_req: ClaimReq,
) -> Result<(H256, ClaimCotaNFTV2Entries), Error> {
    let claims = claim_req.claims;
    let claims_len = claims.len();
    if claims_len == 0 {
        return Err(Error::RequestParamNotFound("claims".to_string()));
    }
    let cota_id_index_pairs: Vec<([u8; 20], [u8; 4])> = claims
        .iter()
        .map(|claim| (claim.cota_id, claim.token_index))
        .collect();
    let withdraw_lock_hash = blake2b_256(&claim_req.withdrawal_lock_script);
    let sender_withdrawals =
        get_withdrawal_cota_by_lock_hash(withdraw_lock_hash, cota_id_index_pairs.clone())?.0;
    if sender_withdrawals.is_empty() || sender_withdrawals.len() != claims_len {
        return Err(Error::CotaIdAndTokenIndexHasNotWithdrawn);
    }
    let withdrawal_block_number = sender_withdrawals.first().unwrap().block_number;
    let withdrawal_out_point = sender_withdrawals.first().unwrap().out_point;
    if sender_withdrawals[1..]
        .iter()
        .any(|withdrawal| withdrawal.block_number != withdrawal_block_number)
    {
        return Err(Error::WithdrawCotaNFTsNotInOneTx);
    }

    let mut hold_keys: Vec<CotaNFTId> = Vec::new();
    let mut hold_values: Vec<CotaNFTInfo> = Vec::new();
    let mut claim_keys: Vec<ClaimCotaNFTKey> = Vec::new();
    let mut key_vec: Vec<(H256, u8)> = Vec::new();
    let mut claim_values: Vec<Byte32> = Vec::new();
    let mut claim_update_leaves: Vec<(H256, H256)> = Vec::with_capacity(claims_len * 2);
    let mut previous_leaves: Vec<(H256, H256)> = Vec::with_capacity(claims_len * 2);

    for withdrawal in sender_withdrawals {
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

        let (hold_key, key) = generate_hold_key(cota_id, token_index);
        let (hold_value, value) = generate_hold_value(configure, state, characteristic);
        hold_keys.push(hold_key);
        hold_values.push(hold_value);
        claim_update_leaves.push((key, value));
        previous_leaves.push((key, H256::zero()));

        let (claim_key, key) = generate_claim_key(cota_id, token_index, out_point);
        claim_keys.push(claim_key);
        key_vec.push((key, version));
    }

    for (key, version) in key_vec {
        let (claim_value, value) = generate_claim_value(version);
        claim_values.push(claim_value);
        claim_update_leaves.push((key, value));
        previous_leaves.push((key, H256::zero()));
    }

    let claim_smt_root = get_cota_smt_root(&claim_req.lock_script).await?;

    let claim_lock_hash = blake2b_256(&claim_req.lock_script);
    let transaction = &StoreTransaction::new(db.transaction());
    let mut claim_smt = init_smt(transaction, claim_lock_hash)?;

    // Add lock to claim smt
    with_lock(claim_lock_hash, || {
        generate_history_smt(&mut claim_smt, claim_lock_hash, claim_smt_root)?;
        claim_smt
            .update_all(claim_update_leaves.clone())
            .map_err(|e| Error::SMTError(e.to_string()))?;
        claim_smt.save_root_and_leaves(previous_leaves.clone())?;
        claim_smt.commit()
    })?;

    let claim_merkle_proof = claim_smt
        .merkle_proof(claim_update_leaves.iter().map(|leave| leave.0).collect())
        .map_err(|e| {
            error!("Claim SMT proof error: {:?}", e.to_string());
            Error::SMTProofError("Claim".to_string())
        })?;
    let claim_merkle_proof_compiled = claim_merkle_proof
        .compile(claim_update_leaves.clone())
        .map_err(|e| {
            error!("Claim SMT proof error: {:?}", e.to_string());
            Error::SMTProofError("Claim".to_string())
        })?;

    let merkel_proof_vec: Vec<u8> = claim_merkle_proof_compiled.into();
    let claim_proof = BytesBuilder::default()
        .extend(merkel_proof_vec.iter().map(|v| Byte::from(*v)))
        .build();

    let withdraw_info = get_withdraw_info(withdrawal_block_number, withdrawal_out_point)?;
    let withdraw_leaf_proof =
        parse_withdraw_witness(withdraw_info.witnesses, &cota_id_index_pairs)?;

    let mut action_vec: Vec<u8> = Vec::new();
    action_vec.extend("Claim ".as_bytes());
    action_vec.extend(claims_len.to_string().as_bytes());
    action_vec.extend(" NFTs".as_bytes());
    let action_bytes = BytesBuilder::default()
        .set(action_vec.iter().map(|v| Byte::from(*v)).collect())
        .build();

    let claim_entries = ClaimCotaNFTV2EntriesBuilder::default()
        .hold_keys(HoldCotaNFTKeyVecBuilder::default().set(hold_keys).build())
        .hold_values(
            HoldCotaNFTValueVecBuilder::default()
                .set(hold_values)
                .build(),
        )
        .claim_keys(ClaimCotaNFTKeyVecBuilder::default().set(claim_keys).build())
        .claim_values(
            ClaimCotaNFTValueVecBuilder::default()
                .set(claim_values)
                .build(),
        )
        .proof(claim_proof)
        .action(action_bytes)
        .withdrawal_proof(withdraw_leaf_proof.withdraw_proof)
        .leaf_keys(
            Byte32VecBuilder::default()
                .set(withdraw_leaf_proof.leaf_keys)
                .build(),
        )
        .leaf_values(
            Byte32VecBuilder::default()
                .set(withdraw_leaf_proof.leaf_values)
                .build(),
        )
        .raw_tx(withdraw_info.raw_tx)
        .tx_proof(withdraw_info.tx_proof)
        .build();

    Ok((*claim_smt.root(), claim_entries))
}
