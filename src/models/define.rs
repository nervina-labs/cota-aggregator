use super::helper::{establish_connection, parse_lock_hash};
use crate::models::block::get_syncer_tip_block_number_with_conn;
use crate::models::helper::{SqlConnection, PAGE_SIZE};
use crate::models::DBResult;
use crate::schema::define_cota_nft_kv_pairs::dsl::*;
use crate::utils::error::Error;
use crate::utils::helper::{diff_time, parse_bytes_n};
use chrono::prelude::*;
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
    let start_time = Local::now().timestamp_millis();
    let (lock_hash_hex, lock_hash_crc_) = parse_lock_hash(lock_hash_);
    let mut page: i64 = 0;
    let mut defines: Vec<DefineDb> = Vec::new();
    loop {
        let defines_page = define_cota_nft_kv_pairs
            .select(get_selection())
            .filter(lock_hash_crc.eq(lock_hash_crc_))
            .filter(lock_hash.eq(lock_hash_hex.clone()))
            .limit(PAGE_SIZE)
            .offset(PAGE_SIZE * page)
            .load::<DefineCotaNft>(conn)
            .map_or_else(
                |e| {
                    error!("Query define error: {}", e.to_string());
                    Err(Error::DatabaseQueryError(e.to_string()))
                },
                |defines| Ok(parse_define_cota_nft(defines)),
            )?;
        let length = defines_page.len();
        defines.extend(defines_page);
        if length < (PAGE_SIZE as usize) {
            break;
        }
        page += 1;
    }
    let block_height = get_syncer_tip_block_number_with_conn(conn)?;
    diff_time(start_time, "SQL get_define_cota_by_lock_hash");
    Ok((defines, block_height))
}

pub fn get_define_cota_by_lock_hash_and_cota_id(
    lock_hash_: [u8; 32],
    cota_id_: [u8; 20],
) -> Result<Option<DefineDb>, Error> {
    let start_time = Local::now().timestamp_millis();
    let conn = &establish_connection();
    let (lock_hash_hex, lock_hash_crc_) = parse_lock_hash(lock_hash_);
    let cota_id_hex = hex::encode(cota_id_);
    let defines: Vec<DefineDb> = define_cota_nft_kv_pairs
        .select(get_selection())
        .filter(lock_hash_crc.eq(lock_hash_crc_))
        .filter(lock_hash.eq(lock_hash_hex))
        .filter(cota_id.eq(cota_id_hex))
        .limit(1)
        .load::<DefineCotaNft>(conn)
        .map_or_else(
            |e| {
                error!("Query define error: {}", e.to_string());
                Err(Error::DatabaseQueryError(e.to_string()))
            },
            |defines| Ok(parse_define_cota_nft(defines)),
        )?;
    diff_time(start_time, "SQL get_define_cota_by_lock_hash_and_cota_id");
    Ok(defines.get(0).map(|v| *v))
}

pub fn get_define_cota_by_cota_id(cota_id_: [u8; 20]) -> Result<Option<DefineDb>, Error> {
    let start_time = Local::now().timestamp_millis();
    let conn = &establish_connection();
    let cota_id_hex = hex::encode(cota_id_);
    let defines: Vec<DefineDb> = define_cota_nft_kv_pairs
        .select(get_selection())
        .filter(cota_id.eq(cota_id_hex))
        .limit(1)
        .load::<DefineCotaNft>(conn)
        .map_or_else(
            |e| {
                error!("Query define error: {}", e.to_string());
                Err(Error::DatabaseQueryError(e.to_string()))
            },
            |defines| Ok(parse_define_cota_nft(defines)),
        )?;
    diff_time(start_time, "SQL get_define_cota_by_cota_id");
    Ok(defines.get(0).cloned())
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

fn get_selection() -> (cota_id, total, issued, configure) {
    (cota_id, total, issued, configure)
}
