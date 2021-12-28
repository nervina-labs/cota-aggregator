use crate::config::load_config;
use crate::db::types::{ClaimDb, DefineDb, HoldDb, ScriptDb, WithdrawDb, WithdrawWithScriptIdDb};
use crate::error::Error;
use crate::utils::{
    generate_crc, parse_bytes, parse_bytes20, parse_bytes32, parse_bytes36, parse_bytes4,
};
use cota_smt::ckb_types::packed::{Byte32, BytesBuilder, Script, ScriptBuilder};
use cota_smt::ckb_types::prelude::*;
use cota_smt::molecule::prelude::Byte;
use lazy_static::lazy_static;
use mysql::prelude::*;
use mysql::*;
use std::sync::Arc;
use std::{result::Result, sync::Mutex};

fn parse_lock_hash(lock_hash: [u8; 32]) -> (String, u32) {
    (hex::encode(lock_hash), generate_crc(&lock_hash))
}

lazy_static! {
    pub static ref CONN: Arc<Mutex<PooledConn>> = {
        let url = load_config().database_url;
        let pool = Pool::new(url).expect("Database pool error");
        Arc::new(Mutex::new(
            pool.get_conn().expect("Database connection error"),
        ))
    };
}

pub fn get_define_cota_by_lock_hash(lock_hash: [u8; 32]) -> Result<Vec<DefineDb>, Error> {
    let (lock_hash_hex, lock_hash_crc) = parse_lock_hash(lock_hash);
    CONN
        .lock()
        .unwrap()
        .query_map(format!("select * from define_cota_nft_kv_pairs where lock_hash = '{}' and lock_hash_crc = '{}'", lock_hash_hex, lock_hash_crc),
                   |(cota_id, total, issued, configure)| DefineDb {
                        cota_id: parse_bytes20(cota_id).unwrap(),
                        total,
                        issued,
                        configure,
                    },
        ).map_err(|_| Error::DatabaseQueryError("define".to_string()))
}

pub fn get_define_cota_by_lock_hash_and_cota_id(
    lock_hash: [u8; 32],
    cota_id: [u8; 20],
) -> Result<Option<DefineDb>, Error> {
    let (lock_hash_hex, lock_hash_crc) = parse_lock_hash(lock_hash);
    let cota_id_hex = hex::encode(cota_id);

    let res = CONN
        .lock()
        .unwrap()
        .query_map(format!("select * from define_cota_nft_kv_pairs where lock_hash = '{}' and lock_hash_crc = '{}' and cota_id = '{}'", lock_hash_hex, lock_hash_crc, cota_id_hex),
                   |(total, issued, configure)| DefineDb {
                       cota_id,
                       total,
                       issued,
                       configure,
                   },
        ).map_err(|_| Error::DatabaseQueryError("define".to_string()))?;
    Ok(res.get(0).map(|v| *v))
}

pub fn get_hold_cota_by_lock_hash(lock_hash: [u8; 32]) -> Result<Vec<HoldDb>, Error> {
    let (lock_hash_hex, lock_hash_crc) = parse_lock_hash(lock_hash);

    CONN
        .lock()
        .unwrap()
        .query_map(format!("select * from hold_cota_nft_kv_pairs where lock_hash = '{}' and lock_hash_crc = '{}'", lock_hash_hex, lock_hash_crc),
                   |(cota_id, token_index, configure, state, characteristic)| HoldDb {
                       cota_id: parse_bytes20(cota_id).unwrap(),
                       token_index: parse_bytes4(token_index).unwrap(),
                       configure,
                       state,
                       characteristic: parse_bytes20(characteristic).unwrap(),
                   },
        ).map_err(|_| Error::DatabaseQueryError("hold".to_string()))
}

pub fn get_withdrawal_cota_by_lock_hash(lock_hash: [u8; 32]) -> Result<Vec<WithdrawDb>, Error> {
    let (lock_hash_hex, lock_hash_crc) = parse_lock_hash(lock_hash);

    let mut conn = CONN.lock().expect("Database connection error");
    let withdrawals_db = conn
        .query_map(format!("select * from withdraw_cota_nft_kv_pairs where lock_hash = '{}' and lock_hash_crc = '{}'", lock_hash_hex, lock_hash_crc),
                   |(cota_id, token_index, configure, state, characteristic, receiver_lock_script_id, out_point)| WithdrawWithScriptIdDb {
                        cota_id: parse_bytes20(cota_id).unwrap(),
                        token_index: parse_bytes4(token_index).unwrap(),
                        configure,
                        state,
                        characteristic: parse_bytes20(characteristic).unwrap(),
                        receiver_lock_script_id,
                        out_point: parse_bytes36(out_point).unwrap(),
            },
        ).map_err(|_| Error::DatabaseQueryError("withdraw".to_string()))?;
    let receiver_lock_script_ids: Vec<String> = withdrawals_db
        .iter()
        .map(|withdrawal| withdrawal.receiver_lock_script_id.to_string())
        .collect();
    let script_id_array = receiver_lock_script_ids.join(",");

    let scripts: Vec<Script> = conn
        .query_map(
            format!("select * from scripts where id in ({})", script_id_array),
            |(id, code_hash, hash_type, args)| ScriptDb {
                id,
                code_hash: parse_bytes32(code_hash).unwrap(),
                hash_type,
                args: parse_bytes(args).unwrap(),
            },
        )
        .map_err(|_| Error::DatabaseQueryError("receiver_lock_script".to_string()))?
        .iter()
        .map(|script_db| {
            let args_bytes: Vec<Byte> = script_db.args.iter().map(|v| Byte::from(*v)).collect();
            ScriptBuilder::default()
                .code_hash(Byte32::from_slice(&script_db.code_hash[..]).unwrap())
                .hash_type(Byte::from(script_db.hash_type))
                .args(BytesBuilder::default().set(args_bytes).build())
                .build()
        })
        .collect();

    let withdraw_db_vec = withdrawals_db
        .iter()
        .zip(scripts.iter())
        .map(|(withdrawal, script)| WithdrawDb {
            cota_id:              withdrawal.cota_id,
            token_index:          withdrawal.token_index,
            configure:            withdrawal.configure,
            state:                withdrawal.state,
            characteristic:       withdrawal.characteristic,
            receiver_lock_script: script.as_slice().to_vec(),
            out_point:            withdrawal.out_point,
        })
        .collect();

    Ok(withdraw_db_vec)
}

pub fn get_claim_cota_by_lock_hash(lock_hash: [u8; 32]) -> Result<Vec<ClaimDb>, Error> {
    let (lock_hash_hex, lock_hash_crc) = parse_lock_hash(lock_hash);
    CONN
        .lock()
        .unwrap()
        .query_map(format!("select * from claim_cota_nft_kv_pairs where lock_hash = '{}' and lock_hash_crc = '{}'", lock_hash_hex, lock_hash_crc),
                   |(cota_id, token_index, out_point)| ClaimDb {
                       cota_id: parse_bytes20(cota_id).unwrap(),
                       token_index: parse_bytes4(token_index).unwrap(),
                       out_point: parse_bytes36(out_point).unwrap(),
                   },
        ).map_err(|_| Error::DatabaseQueryError("claim".to_string()))
}

pub fn get_all_cota_by_lock_hash(
    lock_hash: [u8; 32],
) -> Result<(Vec<DefineDb>, Vec<HoldDb>, Vec<WithdrawDb>, Vec<ClaimDb>), Error> {
    let defines = get_define_cota_by_lock_hash(lock_hash)?;
    let holds = get_hold_cota_by_lock_hash(lock_hash)?;
    let withdrawals = get_withdrawal_cota_by_lock_hash(lock_hash)?;
    let claims = get_claim_cota_by_lock_hash(lock_hash)?;
    Ok((defines, holds, withdrawals, claims))
}
