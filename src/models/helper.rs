use crc::{Crc, CRC_32_ISO_HDLC};
use diesel::mysql::MysqlConnection;
use diesel::r2d2::{self, ConnectionManager, PooledConnection};
use dotenv::dotenv;
use std::env;

pub type SqlConnection = PooledConnection<ConnectionManager<MysqlConnection>>;

pub fn establish_connection() -> SqlConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<MysqlConnection>::new(database_url);
    let pool = r2d2::Pool::new(manager).unwrap();
    pool.get().expect("Error connecting to database")
}

fn generate_crc(v: &[u8]) -> u32 {
    const CRC: Crc<u32> = Crc::<u32>::new(&CRC_32_ISO_HDLC);
    CRC.checksum(v)
}

pub fn parse_lock_hash(lock_hash: [u8; 32]) -> (String, u32) {
    (
        hex::encode(lock_hash),
        generate_crc(hex::encode(lock_hash).as_bytes()),
    )
}

pub fn parse_cota_id_and_token_index_pairs(
    pairs: Vec<([u8; 20], [u8; 4])>,
) -> (Vec<String>, Vec<u32>) {
    let cota_id_hexes: Vec<String> = pairs.iter().map(|pair| hex::encode(pair.0)).collect();
    let token_index_hexes: Vec<u32> = pairs
        .iter()
        .map(|pair| u32::from_be_bytes(pair.1))
        .collect();
    (cota_id_hexes, token_index_hexes)
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
}
