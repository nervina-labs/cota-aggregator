use crate::db::mysql::get_define_cota_by_lock_hash_and_cota_id;
use crate::db::types::DefineDb;
use crate::error::Error;
use crate::request::mint::{MintReq, MintWithdrawal};
use crate::smt::common::generate_history_smt;
use crate::smt::common::{
    generate_define_key, generate_define_value, generate_withdrawal_key, generate_withdrawal_value,
};
use cota_smt::common::*;
use cota_smt::mint::MintCotaNFTEntriesBuilder;
use cota_smt::molecule::prelude::*;
use cota_smt::smt::{Blake2bHasher, H256};
use jsonrpc_http_server::jsonrpc_core::serde_json::Map;
use jsonrpc_http_server::jsonrpc_core::Value;

pub fn generate_mint_smt(mint_req: MintReq) -> Result<Map<String, Value>, Error> {
    let withdrawals = mint_req.withdrawals;
    let withdrawals_len = withdrawals.len();
    if withdrawals_len == 0 {
        return Err(Error::RequestParamNotFound("withdrawals".to_string()));
    }
    let db_define = get_define_cota_by_lock_hash_and_cota_id(mint_req.lock_hash, mint_req.cota_id);
    if db_define.is_none() {
        let cota_id_hex = hex::encode(mint_req.cota_id);
        return Err(Error::CotaIdHasNotDefined(cota_id_hex));
    }
    let mut define_keys: Vec<DefineCotaNFTId> = Vec::new();
    let mut define_old_values: Vec<DefineCotaNFTValue> = Vec::new();
    let mut define_new_values: Vec<DefineCotaNFTValue> = Vec::new();
    let mut withdrawal_keys: Vec<CotaNFTId> = Vec::new();
    let mut withdrawal_values: Vec<WithdrawalCotaNFTValue> = Vec::new();
    let mut smt = generate_history_smt(mint_req.lock_hash);
    let mut update_leaves: Vec<(H256, H256)> = Vec::with_capacity(withdrawals_len + 1);
    let DefineDb {
        cota_id,
        total,
        issued,
        configure,
    } = db_define.unwrap();
    let (define_key, key) = generate_define_key(cota_id);
    define_keys.push(define_key);

    let (define_old_value, _) =
        generate_define_value(total.to_be_bytes(), issued.to_be_bytes(), configure);
    define_old_values.push(define_old_value);

    let new_issued = issued + withdrawals_len as u32;
    let (define_new_value, value) =
        generate_define_value(total.to_be_bytes(), new_issued.to_be_bytes(), configure);
    define_new_values.push(define_new_value);

    update_leaves.push((key, value));
    smt.update(key, value).expect("SMT update leave error");

    let mut action_vec: Vec<u8> = Vec::new();
    if withdrawals_len == 1 {
        action_vec.extend("Mint an NFT ".as_bytes());
        action_vec.extend(&cota_id);
        action_vec.extend(" to ".as_bytes());
        action_vec.extend(&withdrawals.get(0).unwrap().to_lock_script);
    }

    for MintWithdrawal {
        token_index,
        state,
        characteristic,
        to_lock_script,
    } in withdrawals
    {
        let (withdrawal_key, key) = generate_withdrawal_key(cota_id, token_index);
        withdrawal_keys.push(withdrawal_key);

        let (withdrawal_value, value) = generate_withdrawal_value(
            configure,
            state,
            characteristic,
            to_lock_script,
            mint_req.out_point,
        );
        withdrawal_values.push(withdrawal_value);

        update_leaves.push((key, value));
        smt.update(key, value).expect("SMT update leave error");
    }
    let root_hash = smt.root().clone();
    let mut root_hash_bytes = [0u8; 32];
    root_hash_bytes.copy_from_slice(root_hash.as_slice());
    let root_hash_hex = hex::encode(root_hash_bytes);

    println!("smt root hash: {:?}", root_hash_hex);

    let mint_merkle_proof = smt
        .merkle_proof(update_leaves.iter().map(|leave| leave.0).collect())
        .unwrap();
    let mint_merkle_proof_compiled = mint_merkle_proof.compile(update_leaves.clone()).unwrap();
    let verify_result = mint_merkle_proof_compiled
        .verify::<Blake2bHasher>(&root_hash, update_leaves.clone())
        .expect("smt proof verify failed");
    assert!(verify_result, "smt proof verify failed");

    let merkel_proof_vec: Vec<u8> = mint_merkle_proof_compiled.into();
    let merkel_proof_bytes = BytesBuilder::default()
        .extend(merkel_proof_vec.iter().map(|v| Byte::from(*v)))
        .build();

    let action_bytes = BytesBuilder::default()
        .set(action_vec.iter().map(|v| Byte::from(*v)).collect())
        .build();

    let mint_entries = MintCotaNFTEntriesBuilder::default()
        .define_keys(
            DefineCotaNFTKeyVecBuilder::default()
                .set(define_keys)
                .build(),
        )
        .define_old_values(
            DefineCotaNFTValueVecBuilder::default()
                .set(define_old_values)
                .build(),
        )
        .define_new_values(
            DefineCotaNFTValueVecBuilder::default()
                .set(define_new_values)
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

    let mint_entries_hex = hex::encode(mint_entries.as_slice());

    println!("mint_entries_hex: {:?}", mint_entries_hex);

    let mut result = Map::new();
    result.insert("smt_root_hash".to_string(), Value::String(root_hash_hex));
    result.insert(
        "mint_smt_entries".to_string(),
        Value::String(mint_entries_hex),
    );
    Ok(result)
}
