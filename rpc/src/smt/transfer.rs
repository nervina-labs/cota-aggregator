use crate::db::mysql::get_withdrawal_cota_by_lock_hash;
use crate::db::types::WithdrawDb;
use crate::error::Error;
use crate::request::transfer::TransferReq;
use crate::smt::common::{
    generate_claim_key, generate_claim_value, generate_empty_value, generate_history_smt,
    generate_hold_key, generate_hold_value, generate_withdrawal_key, generate_withdrawal_value,
};
use cota_smt::common::*;
use cota_smt::molecule::prelude::*;
use cota_smt::smt::{Blake2bHasher, H256};
use cota_smt::transfer::WithdrawalCotaNFTEntriesBuilder;
use jsonrpc_http_server::jsonrpc_core::serde_json::Map;
use jsonrpc_http_server::jsonrpc_core::Value;
use log::info;

pub fn generate_transfer_smt(transfer_req: TransferReq) -> Result<Map<String, Value>, Error> {
    let transfers = transfer_req.clone().transfers;
    let transfers_len = transfers.len();
    if transfers_len == 0 {
        return Err(Error::RequestParamNotFound("transfers".to_string()));
    }
    let cota_id_and_token_index_pairs = Some(
        transfers
            .iter()
            .map(|transfer| (transfer.cota_id, transfer.token_index))
            .collect(),
    );
    let sender_withdrawals = get_withdrawal_cota_by_lock_hash(
        transfer_req.withdrawal_lock_hash,
        cota_id_and_token_index_pairs,
    )?;
    if sender_withdrawals.is_empty() || sender_withdrawals.len() != transfers_len {
        return Err(Error::CotaIdAndTokenIndexHasNotWithdrawn);
    }

    let mut action_vec: Vec<u8> = Vec::new();
    if transfers_len == 1 {
        action_vec.extend("Transfer an NFT ".as_bytes());
        action_vec.extend(&sender_withdrawals.get(0).unwrap().cota_id);
        action_vec.extend(" to ".as_bytes());
        action_vec.extend(&transfers.get(0).unwrap().to_lock_script);
    }
    let action_bytes = BytesBuilder::default()
        .set(action_vec.iter().map(|v| Byte::from(*v)).collect())
        .build();

    let mut hold_keys: Vec<CotaNFTId> = Vec::new();
    let mut hold_values: Vec<CotaNFTInfo> = Vec::new();
    let mut withdrawal_keys: Vec<CotaNFTId> = Vec::new();
    let mut withdrawal_values: Vec<WithdrawalCotaNFTValue> = Vec::new();
    let mut update_leaves: Vec<(H256, H256)> = Vec::with_capacity(transfers_len * 2);
    let mut smt = generate_history_smt(transfer_req.lock_hash)?;
    for (withdrawal_db, transfer) in sender_withdrawals.into_iter().zip(transfers.clone()) {
        let WithdrawDb {
            cota_id,
            token_index,
            characteristic,
            state,
            configure,
            out_point,
            ..
        } = withdrawal_db;
        let (hold_key, key) = generate_hold_key(cota_id, token_index);
        let (hold_value, _) = generate_hold_value(configure, state, characteristic);
        hold_keys.push(hold_key);
        hold_values.push(hold_value);
        update_leaves.push((key, generate_empty_value().1));

        let (_, key) = generate_claim_key(cota_id, token_index, out_point);
        let (_, value) = generate_claim_value();
        smt.update(key, value)
            .expect("claim SMT update leave error");

        let (withdrawal_key, key) = generate_withdrawal_key(cota_id, token_index);
        withdrawal_keys.push(withdrawal_key);
        let (withdrawal_value, value) = generate_withdrawal_value(
            configure,
            state,
            characteristic,
            transfer.to_lock_script,
            transfer_req.claim_out_point,
        );
        withdrawal_values.push(withdrawal_value);
        update_leaves.push((key, value));
        smt.update(key, value)
            .expect("withdraw SMT update leave error");
    }

    let root_hash = smt.root().clone();
    let mut root_hash_bytes = [0u8; 32];
    root_hash_bytes.copy_from_slice(root_hash.as_slice());
    let withdraw_root_hash_hex = hex::encode(root_hash_bytes);

    info!("transfer_smt_root_hash: {:?}", withdraw_root_hash_hex);

    let withdrawal_merkle_proof = smt
        .merkle_proof(update_leaves.iter().map(|leave| leave.0).collect())
        .unwrap();
    let withdrawal_merkle_proof_compiled = withdrawal_merkle_proof
        .compile(update_leaves.clone())
        .unwrap();
    withdrawal_merkle_proof_compiled
        .verify::<Blake2bHasher>(&root_hash, update_leaves.clone())
        .expect("withdraw smt proof verify failed");

    let merkel_proof_vec: Vec<u8> = withdrawal_merkle_proof_compiled.into();
    let merkel_proof_bytes = BytesBuilder::default()
        .extend(merkel_proof_vec.iter().map(|v| Byte::from(*v)))
        .build();

    let withdrawal_entries = WithdrawalCotaNFTEntriesBuilder::default()
        .hold_keys(HoldCotaNFTKeyVecBuilder::default().set(hold_keys).build())
        .hold_values(
            HoldCotaNFTValueVecBuilder::default()
                .set(hold_values)
                .build(),
        )
        .withdrawal_keys(
            WithdrawalCotaNFTKeyVecBuilder::default()
                .set(withdrawal_keys)
                .build(),
        )
        .withdrawal_values(
            WithdrawalCotaNFTValueVecBuilder::default()
                .set(withdrawal_values)
                .build(),
        )
        .proof(merkel_proof_bytes)
        .action(action_bytes)
        .build();

    let withdrawal_entries_hex = hex::encode(withdrawal_entries.as_slice());

    info!("transfer_smt_entry: {:?}", withdrawal_entries_hex);

    let mut result = Map::new();
    result.insert(
        "smt_root_hash".to_string(),
        Value::String(withdraw_root_hash_hex),
    );
    result.insert(
        "transfer_smt_entry".to_string(),
        Value::String(withdrawal_entries_hex),
    );

    Ok(result)
}
