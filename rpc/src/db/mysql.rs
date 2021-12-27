use crate::config::load_config;
use crate::db::types::{ClaimDb, DefineDb, HoldDb, WithdrawDb};
use crate::utils::{generate_crc, parse_bytes, parse_bytes20, parse_bytes36, parse_bytes4};
use lazy_static::lazy_static;
use mysql::prelude::*;
use mysql::*;
use std::sync::Mutex;

fn parse_lock_hash(lock_hash: [u8; 32]) -> (String, u32) {
    (hex::encode(lock_hash), generate_crc(&lock_hash))
}

lazy_static! {
    pub static ref CONN: Mutex<PooledConn> = {
        let url = load_config().database_url;
        let pool = Pool::new(url).expect("Database pool error");
        Mutex::new(pool.get_conn().expect("Database connection error"))
    };
}

pub fn get_define_cota_by_lock_hash(lock_hash: [u8; 32]) -> Vec<DefineDb> {
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
        ).expect("Query define data error")
}

pub fn get_define_cota_by_lock_hash_and_cota_id(
    lock_hash: [u8; 32],
    cota_id: [u8; 20],
) -> Option<DefineDb> {
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
        ).expect("Query define data error");
    res.get(0).map(|v| *v)
}

pub fn get_hold_cota_by_lock_hash(lock_hash: [u8; 32]) -> Vec<HoldDb> {
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
        ).expect("Query hold data error")
}

pub fn get_withdrawal_cota_by_lock_hash(lock_hash: [u8; 32]) -> Vec<WithdrawDb> {
    let (lock_hash_hex, lock_hash_crc) = parse_lock_hash(lock_hash);

    CONN
        .lock()
        .unwrap()
        .query_map(format!("select * from withdraw_cota_nft_kv_pairs where lock_hash = '{}' and lock_hash_crc = '{}'", lock_hash_hex, lock_hash_crc),
                   |(cota_id, token_index, configure, state, characteristic, receiver_lock_script, out_point)| WithdrawDb {
                        cota_id: parse_bytes20(cota_id).unwrap(),
                        token_index: parse_bytes4(token_index).unwrap(),
                        configure,
                        state,
                        characteristic: parse_bytes20(characteristic).unwrap(),
                        receiver_lock_script: parse_bytes(receiver_lock_script).unwrap(),
                        out_point: parse_bytes36(out_point).unwrap(),
            },
        ).expect("Query withdrawal data error")
}

pub fn get_claim_cota_by_lock_hash(lock_hash: [u8; 32]) -> Vec<ClaimDb> {
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
        ).expect("Query claim data error")
}

pub fn get_all_cota_by_lock_hash(
    lock_hash: [u8; 32],
) -> (Vec<DefineDb>, Vec<HoldDb>, Vec<WithdrawDb>, Vec<ClaimDb>) {
    let defines = get_define_cota_by_lock_hash(lock_hash);
    let holds = get_hold_cota_by_lock_hash(lock_hash);
    let withdrawals = get_withdrawal_cota_by_lock_hash(lock_hash);
    let claims = get_claim_cota_by_lock_hash(lock_hash);
    (defines, holds, withdrawals, claims)
}
