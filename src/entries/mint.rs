use crate::entries::helper::{generate_define_key, generate_define_value};
use crate::entries::helper::{generate_withdrawal_key_v1, generate_withdrawal_value_v1};
use crate::entries::smt::{generate_history_smt, save_smt_root_and_leaves};
use crate::models::define::{get_define_cota_by_lock_hash_and_cota_id, DefineDb};
use crate::request::mint::{MintReq, MintWithdrawal};
use crate::smt::db::cota_db::CotaRocksDB;
use crate::utils::error::Error;
use crate::utils::helper::diff_time;
use chrono::prelude::*;
use cota_smt::common::*;
use cota_smt::mint::MintCotaNFTV1EntriesBuilder;
use cota_smt::molecule::prelude::*;
use cota_smt::smt::{blake2b_256, H256};
use log::error;

pub async fn generate_mint_smt(mint_req: MintReq) -> Result<(String, String), Error> {
    let withdrawals = mint_req.withdrawals.clone();
    let withdrawals_len = withdrawals.len();
    if withdrawals_len == 0 {
        return Err(Error::RequestParamNotFound("withdrawals".to_string()));
    }
    let db_define = get_define_cota_by_lock_hash_and_cota_id(
        blake2b_256(&mint_req.lock_script),
        mint_req.cota_id,
    )?;
    if db_define.is_none() {
        let cota_id_hex = hex::encode(mint_req.cota_id);
        return Err(Error::CotaIdHasNotDefined(cota_id_hex));
    }
    let mut define_keys: Vec<DefineCotaNFTId> = Vec::new();
    let mut define_old_values: Vec<DefineCotaNFTValue> = Vec::new();
    let mut define_new_values: Vec<DefineCotaNFTValue> = Vec::new();
    let mut withdrawal_keys: Vec<WithdrawalCotaNFTKeyV1> = Vec::new();
    let mut withdrawal_values: Vec<WithdrawalCotaNFTValueV1> = Vec::new();
    let db = CotaRocksDB::default();
    let mut smt = generate_history_smt(&db, mint_req.lock_script.clone()).await?;
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
    smt.update(key, value).expect("mint SMT update leave error");

    let mut action_vec: Vec<u8> = Vec::new();
    if withdrawals_len == 1 {
        action_vec.extend("Mint the NFT ".as_bytes());
        action_vec.extend(&cota_id);
        action_vec.extend(&withdrawals.first().unwrap().token_index);
        action_vec.extend(" to ".as_bytes());
        action_vec.extend(&withdrawals.first().unwrap().to_lock_script);
    }

    let start_time = Local::now().timestamp_millis();
    for MintWithdrawal {
        token_index,
        state,
        characteristic,
        to_lock_script,
    } in withdrawals
    {
        let (withdrawal_key, key) =
            generate_withdrawal_key_v1(cota_id, token_index, mint_req.out_point);
        withdrawal_keys.push(withdrawal_key);

        let (withdrawal_value, value) =
            generate_withdrawal_value_v1(configure, state, characteristic, to_lock_script);
        withdrawal_values.push(withdrawal_value);

        update_leaves.push((key, value));
        smt.update(key, value).expect("mint SMT update leave error");
    }
    diff_time(start_time, "Generate mint smt object with update leaves");

    let root_hash = hex::encode(smt.root().as_slice());

    let start_time = Local::now().timestamp_millis();
    save_smt_root_and_leaves(&smt, "Mint", Some(update_leaves.clone()))?;
    let mint_merkle_proof = smt
        .merkle_proof(update_leaves.iter().map(|leave| leave.0).collect())
        .map_err(|e| {
            error!("Mint SMT proof error: {:?}", e.to_string());
            Error::SMTProofError("Mint".to_string())
        })?;
    let mint_merkle_proof_compiled =
        mint_merkle_proof
            .compile(update_leaves.clone())
            .map_err(|e| {
                error!("Mint SMT proof error: {:?}", e.to_string());
                Error::SMTProofError("Mint".to_string())
            })?;
    diff_time(start_time, "Generate mint smt proof");

    let merkel_proof_vec: Vec<u8> = mint_merkle_proof_compiled.into();
    let merkel_proof_bytes = BytesBuilder::default()
        .extend(merkel_proof_vec.iter().map(|v| Byte::from(*v)))
        .build();

    let action_bytes = BytesBuilder::default()
        .set(action_vec.iter().map(|v| Byte::from(*v)).collect())
        .build();

    let mint_entries = MintCotaNFTV1EntriesBuilder::default()
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
            WithdrawalCotaNFTKeyV1VecBuilder::default()
                .set(withdrawal_keys)
                .build(),
        )
        .withdrawal_values(
            WithdrawalCotaNFTValueV1VecBuilder::default()
                .set(withdrawal_values)
                .build(),
        )
        .proof(merkel_proof_bytes)
        .action(action_bytes)
        .build();

    let mint_entry = hex::encode(mint_entries.as_slice());

    Ok((root_hash, mint_entry))
}
