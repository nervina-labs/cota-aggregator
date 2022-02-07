use crate::error::Error;
use crate::models::{establish_connection, parse_lock_hash};
use crate::schema::define_cota_nft_kv_pairs::dsl::*;
use crate::utils::parse_bytes_n;
use diesel::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Queryable, Debug)]
struct DefineCotaNft {
    pub cota_id:   String,
    pub total:     u32,
    pub issued:    u32,
    pub configure: u8,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct DefineDb {
    pub cota_id:   [u8; 20],
    pub total:     u32,
    pub issued:    u32,
    pub configure: u8,
}

pub fn get_define_cota_by_lock_hash(lock_hash_: [u8; 32]) -> Result<Vec<DefineDb>, Error> {
    let conn = &establish_connection();
    let (lock_hash_hex, lock_hash_crc_) = parse_lock_hash(lock_hash_);
    define_cota_nft_kv_pairs
        .select((cota_id, total, issued, configure))
        .filter(lock_hash_crc.eq(lock_hash_crc_))
        .filter(lock_hash.eq(lock_hash_hex))
        .load::<DefineCotaNft>(conn)
        .map_or_else(
            |e| Err(Error::DatabaseQueryError(e.to_string())),
            |defines| {
                Ok(defines
                    .into_iter()
                    .map(|define| DefineDb {
                        cota_id:   parse_bytes_n::<20>(define.cota_id).unwrap(),
                        total:     define.total,
                        issued:    define.issued,
                        configure: define.configure,
                    })
                    .collect())
            },
        )
}

pub fn get_define_cota_by_lock_hash_and_cota_id(
    lock_hash_: [u8; 32],
    cota_id_: [u8; 20],
) -> Result<Option<DefineDb>, Error> {
    let conn = &establish_connection();
    let (lock_hash_hex, lock_hash_crc_) = parse_lock_hash(lock_hash_);
    let cota_id_hex = hex::encode(cota_id_);
    let defines: Vec<DefineDb> = define_cota_nft_kv_pairs
        .select((cota_id, total, issued, configure))
        .filter(lock_hash_crc.eq(lock_hash_crc_))
        .filter(lock_hash.eq(lock_hash_hex))
        .filter(cota_id.eq(cota_id_hex))
        .load::<DefineCotaNft>(conn)
        .map_or_else(
            |e| Err(Error::DatabaseQueryError(e.to_string())),
            |defines| {
                Ok(defines
                    .into_iter()
                    .map(|define| DefineDb {
                        cota_id:   parse_bytes_n::<20>(define.cota_id).unwrap(),
                        total:     define.total,
                        issued:    define.issued,
                        configure: define.configure,
                    })
                    .collect())
            },
        )?;
    Ok(defines.get(0).map(|v| *v))
}
