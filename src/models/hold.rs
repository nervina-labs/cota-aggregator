use super::get_conn;
use super::helper::{parse_cota_id_index_pairs, parse_lock_hash};
use crate::models::block::get_syncer_tip_block_number;
use crate::models::helper::PAGE_SIZE;
use crate::models::{DBResult, DBTotalResult};
use crate::schema::hold_cota_nft_kv_pairs::dsl::hold_cota_nft_kv_pairs;
use crate::schema::hold_cota_nft_kv_pairs::*;
use crate::utils::error::Error;
use crate::utils::helper::{diff_time, parse_bytes_n};
use chrono::prelude::*;
use diesel::*;
use log::error;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Queryable, Debug)]
struct HoldCotaNft {
    pub cota_id:        String,
    pub token_index:    u32,
    pub configure:      u8,
    pub state:          u8,
    pub characteristic: String,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct HoldDb {
    pub cota_id:        [u8; 20],
    pub token_index:    [u8; 4],
    pub configure:      u8,
    pub state:          u8,
    pub characteristic: [u8; 20],
}

pub fn get_hold_cota_by_lock_hash(
    lock_hash_: [u8; 32],
    cota_id_index_pairs: &[([u8; 20], [u8; 4])],
) -> DBResult<HoldDb> {
    let start_time = Local::now().timestamp_millis();
    let conn = &get_conn();
    let (lock_hash_hex, lock_hash_crc_) = parse_lock_hash(lock_hash_);
    let mut hold_vec: Vec<HoldDb> = vec![];
    let holds: Vec<HoldDb> = match cota_id_index_pairs.len() {
        0 => {
            let mut page: i64 = 0;
            loop {
                let holds_page: Vec<HoldDb> = hold_cota_nft_kv_pairs
                    .select(get_selection())
                    .filter(lock_hash_crc.eq(lock_hash_crc_))
                    .filter(lock_hash.eq(lock_hash_hex.clone()))
                    .limit(PAGE_SIZE)
                    .offset(PAGE_SIZE * page)
                    .load::<HoldCotaNft>(conn)
                    .map_or_else(
                        |e| {
                            error!("Query hold error: {}", e.to_string());
                            Err(Error::DatabaseQueryError(e.to_string()))
                        },
                        |holds| Ok(parse_hold_cota_nfts(holds)),
                    )?;
                let length = holds_page.len();
                hold_vec.extend(holds_page);
                if length < (PAGE_SIZE as usize) {
                    break;
                }
                page += 1;
            }
            hold_vec
        }
        _ => {
            let pair_vec = parse_cota_id_index_pairs(cota_id_index_pairs);
            for (cota_id_str, token_index_u32) in pair_vec.into_iter() {
                let holds_: Vec<HoldDb> = hold_cota_nft_kv_pairs
                    .select(get_selection())
                    .filter(lock_hash_crc.eq(lock_hash_crc_))
                    .filter(lock_hash.eq(lock_hash_hex.clone()))
                    .filter(cota_id.eq(cota_id_str))
                    .filter(token_index.eq(token_index_u32))
                    .order(updated_at.desc())
                    .limit(1)
                    .load::<HoldCotaNft>(conn)
                    .map_or_else(
                        |e| {
                            error!("Query hold error: {}", e.to_string());
                            Err(Error::DatabaseQueryError(e.to_string()))
                        },
                        |holds| Ok(parse_hold_cota_nfts(holds)),
                    )?;
                if !holds_.is_empty() {
                    hold_vec.push(holds_[0]);
                }
            }
            hold_vec
        }
    };
    let block_height = get_syncer_tip_block_number()?;
    diff_time(start_time, "SQL get_hold_cota_by_lock_hash");
    Ok((holds, block_height))
}

pub fn get_hold_cota_count_by_lock_hash(
    lock_hash_: [u8; 32],
    cota_id_: [u8; 20],
) -> Result<i64, Error> {
    let start_time = Local::now().timestamp_millis();
    let (lock_hash_hex, lock_hash_crc_) = parse_lock_hash(lock_hash_);
    let cota_id_str = hex::encode(cota_id_);
    let hold_count: i64 = hold_cota_nft_kv_pairs
        .filter(lock_hash_crc.eq(lock_hash_crc_))
        .filter(lock_hash.eq(lock_hash_hex.clone()))
        .filter(cota_id.eq(cota_id_str))
        .count()
        .get_result::<i64>(&get_conn())
        .map_err(|e| {
            error!("Query hold error: {}", e.to_string());
            Error::DatabaseQueryError(e.to_string())
        })?;
    diff_time(start_time, "SQL get_hold_cota_count_by_lock_hash");
    Ok(hold_count)
}

pub fn check_hold_cota_by_lock_hash(
    lock_hash_: [u8; 32],
    cota_id_and_token_index_pair: ([u8; 20], [u8; 4]),
) -> Result<(bool, u64), Error> {
    let start_time = Local::now().timestamp_millis();
    let (lock_hash_hex, lock_hash_crc_) = parse_lock_hash(lock_hash_);
    let cota_id_str = hex::encode(cota_id_and_token_index_pair.0);
    let token_index_u32 = u32::from_be_bytes(cota_id_and_token_index_pair.1);
    let count = hold_cota_nft_kv_pairs
        .filter(cota_id.eq(cota_id_str))
        .filter(token_index.eq(token_index_u32))
        .filter(lock_hash_crc.eq(lock_hash_crc_))
        .filter(lock_hash.eq(lock_hash_hex.clone()))
        .count()
        .get_result::<i64>(&get_conn())
        .map_err(|e| {
            error!("Check hold cota count error: {:?}", e.to_string());
            Error::DatabaseQueryError("Hold".to_string())
        })?;
    let is_exist = count > 0;
    let block_height = get_syncer_tip_block_number()?;
    diff_time(start_time, "SQL check_hold_cota_by_lock_hash");
    Ok((is_exist, block_height))
}

pub fn get_hold_cota_by_lock_hash_and_page(
    lock_hash_: [u8; 32],
    page: i64,
    page_size: i64,
    cota_id_opt: Option<[u8; 20]>,
) -> DBTotalResult<HoldDb> {
    let start_time = Local::now().timestamp_millis();
    let conn = &get_conn();
    let (lock_hash_hex, lock_hash_crc_) = parse_lock_hash(lock_hash_);
    let total_result = match cota_id_opt {
        Some(cota_id_) => hold_cota_nft_kv_pairs
            .filter(lock_hash_crc.eq(lock_hash_crc_))
            .filter(lock_hash.eq(lock_hash_hex.clone()))
            .filter(cota_id.eq(hex::encode(&cota_id_)))
            .count()
            .get_result::<i64>(conn),
        None => hold_cota_nft_kv_pairs
            .filter(lock_hash_crc.eq(lock_hash_crc_))
            .filter(lock_hash.eq(lock_hash_hex.clone()))
            .filter(cota_id.ne(hex::encode(&[0u8; 20])))
            .count()
            .get_result::<i64>(conn),
    };
    let total: i64 = total_result.map_err(|e| {
        error!("Query hold error: {}", e.to_string());
        Error::DatabaseQueryError(e.to_string())
    })?;
    let block_height: u64 = get_syncer_tip_block_number()?;

    let holds_result = match cota_id_opt {
        Some(cota_id_) => hold_cota_nft_kv_pairs
            .select(get_selection())
            .filter(lock_hash_crc.eq(lock_hash_crc_))
            .filter(lock_hash.eq(lock_hash_hex.clone()))
            .filter(cota_id.eq(hex::encode(&cota_id_)))
            .order(updated_at.desc())
            .limit(page_size)
            .offset(page_size * page)
            .load::<HoldCotaNft>(conn),
        None => hold_cota_nft_kv_pairs
            .select(get_selection())
            .filter(lock_hash_crc.eq(lock_hash_crc_))
            .filter(lock_hash.eq(lock_hash_hex.clone()))
            .filter(cota_id.ne(hex::encode(&[0u8; 20])))
            .order(updated_at.desc())
            .limit(page_size)
            .offset(page_size * page)
            .load::<HoldCotaNft>(conn),
    };
    let holds: Vec<HoldDb> = holds_result.map_or_else(
        |e| {
            error!("Query hold error: {}", e.to_string());
            Err(Error::DatabaseQueryError(e.to_string()))
        },
        |holds| Ok(parse_hold_cota_nfts(holds)),
    )?;
    diff_time(start_time, "SQL get_hold_cota_by_lock_hash_and_page");
    Ok((holds, total, block_height))
}

fn parse_hold_cota_nfts(holds: Vec<HoldCotaNft>) -> Vec<HoldDb> {
    holds.into_iter().map(parse_hold_cota_nft).collect()
}

fn parse_hold_cota_nft(hold: HoldCotaNft) -> HoldDb {
    HoldDb {
        cota_id:        parse_bytes_n::<20>(hold.cota_id).unwrap(),
        token_index:    hold.token_index.to_be_bytes(),
        state:          hold.state,
        configure:      hold.configure,
        characteristic: parse_bytes_n::<20>(hold.characteristic).unwrap(),
    }
}

fn get_selection() -> (cota_id, token_index, configure, state, characteristic) {
    (cota_id, token_index, configure, state, characteristic)
}
