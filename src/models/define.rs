use super::get_conn;
use super::helper::parse_lock_hash;
use crate::models::block::get_syncer_tip_block_number;
use crate::models::helper::PAGE_SIZE;
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
    pub cota_id:      String,
    pub total:        u32,
    pub issued:       u32,
    pub configure:    u8,
    pub block_number: u64,
}

#[derive(Serialize, Debug, Copy, Clone, Eq, PartialEq, Default)]
pub struct DefineDb {
    #[serde(skip_serializing)]
    pub cota_id:      [u8; 20],
    pub total:        u32,
    pub issued:       u32,
    pub configure:    u8,
    #[serde(skip_serializing)]
    pub block_number: u64,
}

pub fn get_define_cota_by_lock_hash(lock_hash_: [u8; 32]) -> DBResult<DefineDb> {
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
            .load::<DefineCotaNft>(&get_conn())
            .map_or_else(
                |e| {
                    error!("Query define error: {}", e.to_string());
                    Err(Error::DatabaseQueryInvalid(e.to_string()))
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
    let block_height = get_syncer_tip_block_number()?;
    diff_time(start_time, "SQL get_define_cota_by_lock_hash");
    Ok((defines, block_height))
}

pub fn get_define_cota_by_lock_hash_and_cota_id(
    lock_hash_: [u8; 32],
    cota_id_: [u8; 20],
) -> Result<Option<DefineDb>, Error> {
    let start_time = Local::now().timestamp_millis();
    let (lock_hash_hex, lock_hash_crc_) = parse_lock_hash(lock_hash_);
    let cota_id_hex = hex::encode(cota_id_);
    let defines: Vec<DefineDb> = define_cota_nft_kv_pairs
        .select(get_selection())
        .filter(lock_hash_crc.eq(lock_hash_crc_))
        .filter(lock_hash.eq(lock_hash_hex))
        .filter(cota_id.eq(cota_id_hex))
        .limit(1)
        .load::<DefineCotaNft>(&get_conn())
        .map_or_else(
            |e| {
                error!("Query define error: {}", e.to_string());
                Err(Error::DatabaseQueryInvalid(e.to_string()))
            },
            |defines| Ok(parse_define_cota_nft(defines)),
        )?;
    diff_time(start_time, "SQL get_define_cota_by_lock_hash_and_cota_id");
    Ok(defines.get(0).copied())
}

pub fn get_define_cota_by_cota_id(cota_id_: [u8; 20]) -> Result<Option<DefineDb>, Error> {
    let start_time = Local::now().timestamp_millis();
    let cota_id_hex = hex::encode(cota_id_);
    let defines: Vec<DefineDb> = define_cota_nft_kv_pairs
        .select(get_selection())
        .filter(cota_id.eq(cota_id_hex))
        .limit(1)
        .load::<DefineCotaNft>(&get_conn())
        .map_or_else(
            |e| {
                error!("Query define error: {}", e.to_string());
                Err(Error::DatabaseQueryInvalid(e.to_string()))
            },
            |defines| Ok(parse_define_cota_nft(defines)),
        )?;
    diff_time(start_time, "SQL get_define_cota_by_cota_id");
    Ok(defines.get(0).cloned())
}

pub fn get_lock_hash_by_cota_id(cota_id_: [u8; 20]) -> Result<[u8; 32], Error> {
    let cota_id_hex = hex::encode(cota_id_);
    define_cota_nft_kv_pairs
        .select(lock_hash)
        .filter(cota_id.eq(cota_id_hex))
        .limit(1)
        .first::<String>(&get_conn())
        .map_or_else(
            |e| {
                error!("Query lock hash by cota id error: {}", e.to_string());
                Err(Error::DatabaseQueryInvalid(e.to_string()))
            },
            |lock_hash_| Ok(parse_bytes_n::<32>(lock_hash_).unwrap()),
        )
}

fn parse_define_cota_nft(defines: Vec<DefineCotaNft>) -> Vec<DefineDb> {
    defines
        .into_iter()
        .map(|define| DefineDb {
            cota_id:      parse_bytes_n::<20>(define.cota_id).unwrap(),
            total:        define.total,
            issued:       define.issued,
            configure:    define.configure,
            block_number: define.block_number,
        })
        .collect()
}

fn get_selection() -> (cota_id, total, issued, configure, block_number) {
    (cota_id, total, issued, configure, block_number)
}
