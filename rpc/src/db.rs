use crate::config::load_config;
use lazy_static::lazy_static;
use mysql::prelude::*;
use mysql::*;
use std::sync::Mutex;

lazy_static! {
    pub static ref CONN: Mutex<PooledConn> = {
        let url = load_config().database;
        let pool = Pool::new(url).expect("Database pool error");
        Mutex::new(pool.get_conn().expect("Database connection error"))
    };
}

pub fn get_registry_lock_hashes() -> Vec<[u8; 32]> {
    let res: Vec<String> = CONN
        .lock()
        .unwrap()
        .query("select lock_hash from register_cota_kv_pairs")
        .expect("Select lock hashes error");

    let mut lock_hash_vec = [0u8; 32];
    let lock_hashes = res
        .iter()
        .map(|hash| {
            let hex_hash = hex::decode(hash).expect("Hex decode error");
            lock_hash_vec.copy_from_slice(&hex_hash);
            lock_hash_vec
        })
        .collect::<Vec<[u8; 32]>>();
    lock_hashes
}
