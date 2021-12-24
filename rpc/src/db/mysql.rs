use crate::config::load_config;
use crate::db::types::{DefineDb, WithdrawDb};
use crate::utils::{generate_crc, parse_bytes20, parse_bytes32, parse_bytes4, parse_bytes72};
use lazy_static::lazy_static;
use mysql::prelude::*;
use mysql::*;
use std::sync::Mutex;

lazy_static! {
    pub static ref CONN: Mutex<PooledConn> = {
        let url = load_config().database_url;
        let pool = Pool::new(url).expect("Database pool error");
        Mutex::new(pool.get_conn().expect("Database connection error"))
    };
}

pub fn get_define_cota_of_lock_hash(lock_hash: [u8; 32]) -> Vec<DefineDb> {
    let lock_hash_hex = hex::encode(lock_hash);
    let lock_hash_crc = generate_crc(&lock_hash);

    let res = CONN
        .lock()
        .unwrap()
        .query_map(format!("select * from define_cota_nft_kv_pairs where lock_hash = '{}' and lock_hash_crc = '{}'", lock_hash_hex, lock_hash_crc),
                   |(cota_id, total, issued, configure)| DefineDb {
                        cota_id: parse_bytes20(cota_id),
                        total,
                        issued,
                        configure,
                    },
        ).expect("Query define data error");
    res
}

pub fn get_withdrawal_cota_of_lock_hash(lock_hash: [u8; 32]) -> Vec<WithdrawDb> {
    let lock_hash_hex = hex::encode(lock_hash);
    let lock_hash_crc = generate_crc(&lock_hash);

    let res = CONN
        .lock()
        .unwrap()
        .query_map(format!("select * from withdraw_cota_nft_kv_pairs where lock_hash = '{}' and lock_hash_crc = '{}'", lock_hash_hex, lock_hash_crc),
                   |(cota_id, token_index, configure, state, characteristic, receiver_lock_hash, out_point)| WithdrawDb {
                        cota_id: parse_bytes20(cota_id),
                        token_index: parse_bytes4(token_index),
                        configure,
                        state,
                        characteristic: parse_bytes20(characteristic),
                        receiver_lock_hash: parse_bytes32(receiver_lock_hash),
                        out_point: parse_bytes72(out_point),
            },
        ).expect("Query define data error");
    res
}
