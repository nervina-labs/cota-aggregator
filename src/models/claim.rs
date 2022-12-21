use super::get_conn;
use super::helper::parse_lock_hash;
use crate::models::block::get_syncer_tip_block_number;
use crate::models::helper::{generate_crc, PAGE_SIZE};
use crate::models::DBResult;
use crate::schema::claimed_cota_nft_kv_pairs::dsl::claimed_cota_nft_kv_pairs;
use crate::schema::claimed_cota_nft_kv_pairs::*;
use crate::utils::error::Error;
use crate::utils::helper::{diff_time, parse_bytes_n};
use chrono::prelude::*;
use diesel::*;
use log::error;

#[derive(Queryable)]
pub struct ClaimCotaNft {
    pub cota_id:     String,
    pub token_index: u32,
    pub out_point:   String,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ClaimDb {
    pub cota_id:     [u8; 20],
    pub token_index: [u8; 4],
    pub out_point:   [u8; 24],
}

pub fn get_claim_cota_by_lock_hash(lock_hash_: [u8; 32]) -> DBResult<ClaimDb> {
    let start_time = Local::now().timestamp_millis();
    let (lock_hash_hex, lock_hash_crc_) = parse_lock_hash(lock_hash_);
    let mut page: i64 = 0;
    let mut claims: Vec<ClaimDb> = Vec::new();
    loop {
        let claims_page: Vec<ClaimDb> = claimed_cota_nft_kv_pairs
            .select(get_selection())
            .filter(lock_hash_crc.eq(lock_hash_crc_))
            .filter(lock_hash.eq(lock_hash_hex.clone()))
            .limit(PAGE_SIZE)
            .offset(PAGE_SIZE * page)
            .load::<ClaimCotaNft>(&get_conn())
            .map_or_else(
                |e| {
                    error!("Query claim error: {}", e.to_string());
                    Err(Error::DatabaseQueryError(e.to_string()))
                },
                |claims| Ok(parse_claimed_cota_nft(claims)),
            )?;
        let length = claims_page.len();
        claims.extend(claims_page);
        if length < (PAGE_SIZE as usize) {
            break;
        }
        page += 1;
    }
    let block_height = get_syncer_tip_block_number()?;
    diff_time(start_time, "SQL get_claim_cota_by_lock_hash");
    Ok((claims, block_height))
}

pub fn is_exist_in_claim(
    lock_hash_: [u8; 32],
    cota_id_: [u8; 20],
    token_index_: [u8; 4],
    out_point_: [u8; 24],
) -> bool {
    let (lock_hash_hex, lock_hash_crc_) = parse_lock_hash(lock_hash_);
    let cota_id_hex = hex::encode(cota_id_);
    let cota_id_crc_u32 = generate_crc(cota_id_hex.as_bytes());
    let token_index_u32 = u32::from_be_bytes(token_index_);
    let out_point_hex = hex::encode(out_point_);
    let out_point_crc_u32 = generate_crc(out_point_hex.as_bytes());
    claimed_cota_nft_kv_pairs
        .filter(lock_hash_crc.eq(lock_hash_crc_))
        .filter(lock_hash.eq(lock_hash_hex.clone()))
        .filter(cota_id_crc.eq(cota_id_crc_u32))
        .filter(cota_id.eq(cota_id_hex))
        .filter(token_index.eq(token_index_u32))
        .filter(out_point_crc.eq(out_point_crc_u32))
        .filter(out_point.eq(out_point_hex))
        .limit(1)
        .count()
        .get_result::<i64>(&get_conn())
        .map_or(false, |count_| count_ > 0)
}

fn parse_claimed_cota_nft(claims: Vec<ClaimCotaNft>) -> Vec<ClaimDb> {
    claims
        .into_iter()
        .map(|claim| ClaimDb {
            cota_id:     parse_bytes_n::<20>(claim.cota_id).unwrap(),
            token_index: claim.token_index.to_be_bytes(),
            out_point:   parse_bytes_n::<24>(claim.out_point).unwrap(),
        })
        .collect()
}

fn get_selection() -> (cota_id, token_index, out_point) {
    (cota_id, token_index, out_point)
}
