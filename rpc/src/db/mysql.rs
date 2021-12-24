use crate::config::load_config;
use crate::db::types::DefineDb;
use crate::utils::generate_crc;
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

fn parse_cota_id(cota_id: String) -> [u8; 20] {
    let mut cota_id_bytes = [0u8; 20];
    let cota_id_vec = hex::decode(cota_id).expect("Parse cota_id hex to bytes error");
    cota_id_bytes.copy_from_slice(&cota_id_vec);
    cota_id_bytes
}

pub fn get_define_cota_of_lock_hash(lock_hash: [u8; 32]) -> Vec<DefineDb> {
    let lock_hash_hex = hex::encode(lock_hash);
    let lock_hash_crc = generate_crc(&lock_hash);

    let res = CONN
        .lock()
        .unwrap()
        .query_map(format!("select * from define_cota_nft_kv_pairs where lock_hash = '{}' and lock_hash_crc = '{}'", lock_hash_hex, lock_hash_crc),
                   |(cota_id, total, issued, configure)| DefineDb {
                        cota_id: parse_cota_id(cota_id),
                        total,
                        issued,
                        configure,
                    },
        ).expect("Query define data error");
    res
}
