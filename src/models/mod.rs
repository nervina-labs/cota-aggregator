use crate::utils::{generate_crc, parse_bytes, parse_bytes_n};
use mysql::*;

pub mod check_info;
pub mod claimed;
pub mod define;
pub mod hold;
pub mod scripts;
mod types;
pub mod withdraw;

use diesel::mysql::MysqlConnection;
use diesel::r2d2::{self, ConnectionManager, PooledConnection};
use dotenv::dotenv;
use std::env;

pub fn establish_connection() -> PooledConnection<ConnectionManager<MysqlConnection>> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<MysqlConnection>::new(database_url);
    let pool = r2d2::Pool::new(manager).unwrap();
    pool.get().expect("Error connecting to database")
}

fn parse_lock_hash(lock_hash: [u8; 32]) -> (String, u32) {
    (
        hex::encode(lock_hash),
        generate_crc(hex::encode(lock_hash).as_bytes()),
    )
}

fn parse_mysql_bytes_n<const N: usize>(v: Value) -> [u8; N] {
    let vec = from_value::<Vec<u8>>(v);
    parse_bytes_n::<N>(String::from_utf8(vec).unwrap()).unwrap()
}

fn parse_mysql_bytes_value(v: Value) -> Vec<u8> {
    let vec = from_value::<Vec<u8>>(v);
    parse_bytes(String::from_utf8(vec).unwrap()).unwrap()
}
