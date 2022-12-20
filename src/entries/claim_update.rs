use crate::ckb::indexer::get_cota_smt_root;
use crate::ckb::rpc::get_withdraw_info;
use crate::entries::helper::{
    generate_claim_key, generate_claim_value, generate_hold_key, generate_hold_value, with_lock,
};
use crate::entries::smt::{generate_history_smt, init_smt};
use crate::entries::witness::parse_witness_withdraw_proof;
use crate::models::claim::is_exist_in_claim;
use crate::models::withdrawal::nft::{get_withdrawal_cota_by_lock_hash, WithdrawDb};
use crate::request::claim::ClaimUpdateReq;
use crate::smt::transaction::store_transaction::StoreTransaction;
use crate::smt::RootSaver;
use crate::utils::error::Error;
use crate::ROCKS_DB;
use cota_smt::common::*;
use cota_smt::molecule::prelude::*;
use cota_smt::smt::{blake2b_256, H256};
use cota_smt::transfer_update::{ClaimUpdateCotaNFTV2Entries, ClaimUpdateCotaNFTV2EntriesBuilder};
use log::error;

pub async fn generate_claim_update_smt(
    claim_update_req: ClaimUpdateReq,
) -> Result<(H256, ClaimUpdateCotaNFTV2Entries, H256), Error> {
    let nfts = claim_update_req.nfts;
    let nfts_len = nfts.len();
    if nfts_len == 0 {
        return Err(Error::RequestParamNotFound("nfts".to_string()));
    }
    let claim_lock_script = claim_update_req.lock_script;
    let cota_id_index_pairs: Vec<([u8; 20], [u8; 4])> = nfts
        .iter()
        .map(|nft| (nft.cota_id, nft.token_index))
        .collect();
    let withdraw_lock_hash = blake2b_256(&claim_update_req.withdrawal_lock_script);
    let sender_withdrawals =
        get_withdrawal_cota_by_lock_hash(withdraw_lock_hash, &cota_id_index_pairs)?.0;
    if sender_withdrawals.is_empty() || sender_withdrawals.len() != nfts_len {
        return Err(Error::CotaIdAndTokenIndexHasNotWithdrawn);
    }
    let is_receiver = sender_withdrawals
        .iter()
        .any(|withdrawal| &withdrawal.receiver_lock_script == &claim_lock_script);
    if !is_receiver {
        return Err(Error::CotaIdAndTokenIndexHasNotWithdrawn);
    }
    let claim_lock_hash = blake2b_256(&claim_lock_script);
    let is_claimed = sender_withdrawals.iter().any(|withdrawal| {
        is_exist_in_claim(
            claim_lock_hash,
            withdrawal.cota_id,
            withdrawal.token_index,
            withdrawal.out_point,
        )
    });
    if is_claimed {
        return Err(Error::CotaIdAndTokenIndexHasNotWithdrawn);
    }
    let withdrawal_block_number = sender_withdrawals.first().unwrap().block_number;
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
    let mut claim_infos: Vec<ClaimCotaNFTInfo> = Vec::new();
    let mut claim_update_leaves: Vec<(H256, H256)> = Vec::with_capacity(nfts_len * 2);
    let mut previous_leaves: Vec<(H256, H256)> = Vec::with_capacity(nfts_len * 2);
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

    let claim_smt_root = get_cota_smt_root(&claim_lock_script).await?;

    let claim_lock_hash = blake2b_256(&claim_lock_script);
    let transaction = &StoreTransaction::new(ROCKS_DB.transaction());
    let mut claim_smt = init_smt(transaction, claim_lock_hash)?;
    // Add lock to smt
    with_lock(claim_lock_hash, || {
        generate_history_smt(&mut claim_smt, claim_lock_hash, claim_smt_root)?;
        claim_smt
            .update_all(claim_update_leaves.clone())
            .map_err(|e| Error::SMTError(e.to_string()))?;
        claim_smt.save_root_and_leaves(previous_leaves.clone())?;
        claim_smt.commit()
    })?;

    let leaf_keys: Vec<H256> = claim_update_leaves.iter().map(|leave| leave.0).collect();
    let claim_update_merkle_proof = claim_smt.merkle_proof(leaf_keys.clone()).map_err(|e| {
        error!("Claim update SMT proof error: {:?}", e.to_string());
        Error::SMTProofError("ClaimUpdate".to_string())
    })?;
    let claim_update_merkle_proof_compiled =
        claim_update_merkle_proof.compile(leaf_keys).map_err(|e| {
            error!("Claim update SMT proof error: {:?}", e.to_string());
            Error::SMTProofError("ClaimUpdate".to_string())
        })?;

    let merkel_proof_vec: Vec<u8> = claim_update_merkle_proof_compiled.into();
    let claim_proof = BytesBuilder::default()
        .extend(merkel_proof_vec.iter().map(|v| Byte::from(*v)))
        .build();

    let withdraw_info = get_withdraw_info(
        withdrawal_block_number,
        claim_update_req.withdrawal_lock_script,
    )
    .await?;
    let withdraw_proof = parse_witness_withdraw_proof(
        withdraw_info.witnesses,
        &cota_id_index_pairs,
        withdraw_info.block_number,
    )?;

    let mut action_vec: Vec<u8> = Vec::new();
    action_vec.extend("Claim ".as_bytes());
    action_vec.extend(nfts_len.to_string().as_bytes());
    action_vec.extend(" NFTs and update NFTs information".as_bytes());
    let action_bytes = BytesBuilder::default()
        .set(action_vec.iter().map(|v| Byte::from(*v)).collect())
        .build();

    let claim_update_entries = ClaimUpdateCotaNFTV2EntriesBuilder::default()
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
        .action(action_bytes)
        .withdrawal_proof(withdraw_proof)
        .leaf_keys(Byte32Vec::default())
        .leaf_values(Byte32Vec::default())
        .raw_tx(withdraw_info.raw_tx)
        .output_index(withdraw_info.output_index)
        .tx_proof(withdraw_info.tx_proof)
        .build();

    Ok((
        *claim_smt.root(),
        claim_update_entries,
        withdraw_info.block_hash,
    ))
}
