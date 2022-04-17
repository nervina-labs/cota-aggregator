use crc::{Crc, CRC_32_ISO_HDLC};
use diesel::mysql::MysqlConnection;
use diesel::r2d2::{self, ConnectionManager, Pool};
use jsonrpc_http_server::jsonrpc_core::serde_json::from_str;
use std::env;

pub type SqlConnection = Pool<ConnectionManager<MysqlConnection>>;

pub const PAGE_SIZE: i64 = 1000;

pub fn establish_connection() -> SqlConnection {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<MysqlConnection>::new(database_url);
    let max: u32 = match env::var("MAX_POOL") {
        Ok(max_) => from_str::<u32>(&max_).unwrap(),
        Err(_e) => 20,
    };
    r2d2::Pool::builder().max_size(max).build(manager).unwrap()
}

pub fn generate_crc(v: &[u8]) -> u32 {
    const CRC: Crc<u32> = Crc::<u32>::new(&CRC_32_ISO_HDLC);
    CRC.checksum(v)
}

pub fn parse_lock_hash(lock_hash: [u8; 32]) -> (String, u32) {
    (
        hex::encode(lock_hash),
        generate_crc(hex::encode(lock_hash).as_bytes()),
    )
}

pub fn parse_cota_id_and_token_index_pairs(pairs: Vec<([u8; 20], [u8; 4])>) -> Vec<(String, u32)> {
    pairs
        .into_iter()
        .map(|pair| (hex::encode(pair.0), u32::from_be_bytes(pair.1)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_crc() {
        assert_eq!(generate_crc("cota".as_bytes()), 985327312u32);
        assert_eq!(
            generate_crc(
                &"41a7a00cced6ecc5be4ec248c01096b705e4cd9d8b0a5ef5cdb6760a3742f5de"
                    .as_bytes()
                    .to_vec()
            ),
            2934249110
        )
    }

    // TODO: Add more tests
}
