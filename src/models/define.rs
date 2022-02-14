use super::helper::{establish_connection, parse_lock_hash};
use crate::models::block::get_syncer_tip_block_number_with_conn;
use crate::models::helper::SqlConnection;
use crate::models::DBResult;
use crate::schema::define_cota_nft_kv_pairs::dsl::*;
use crate::utils::error::Error;
use crate::utils::helper::parse_bytes_n;
use diesel::*;
use log::error;
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

pub fn get_define_cota_by_lock_hash_with_conn(
    conn: &SqlConnection,
    lock_hash_: [u8; 32],
) -> DBResult<DefineDb> {
    let (lock_hash_hex, lock_hash_crc_) = parse_lock_hash(lock_hash_);
    let defines = define_cota_nft_kv_pairs
        .select((cota_id, total, issued, configure))
        .filter(lock_hash_crc.eq(lock_hash_crc_))
        .filter(lock_hash.eq(lock_hash_hex))
        .load::<DefineCotaNft>(conn)
        .map_or_else(
            |e| {
                error!("Query define error: {}", e.to_string());
                Err(Error::DatabaseQueryError(e.to_string()))
            },
            |defines| Ok(parse_define_cota_nft(defines)),
        )?;
    let block_height = get_syncer_tip_block_number_with_conn(conn)?;
    Ok((defines, block_height))
}

pub fn get_define_cota_by_lock_hash(lock_hash_: [u8; 32]) -> DBResult<DefineDb> {
    get_define_cota_by_lock_hash_with_conn(&establish_connection(), lock_hash_)
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
            |e| {
                error!("Query define error: {}", e.to_string());
                Err(Error::DatabaseQueryError(e.to_string()))
            },
            |defines| Ok(parse_define_cota_nft(defines)),
        )?;
    Ok(defines.get(0).map(|v| *v))
}

fn parse_define_cota_nft(defines: Vec<DefineCotaNft>) -> Vec<DefineDb> {
    defines
        .into_iter()
        .map(|define| DefineDb {
            cota_id:   parse_bytes_n::<20>(define.cota_id).unwrap(),
            total:     define.total,
            issued:    define.issued,
            configure: define.configure,
        })
        .collect()
}
