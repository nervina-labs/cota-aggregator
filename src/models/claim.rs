use super::helper::parse_lock_hash;
use crate::models::block::get_syncer_tip_block_number;
use crate::models::helper::PAGE_SIZE;
use crate::models::DBResult;
use crate::schema::claimed_cota_nft_kv_pairs::dsl::claimed_cota_nft_kv_pairs;
use crate::schema::claimed_cota_nft_kv_pairs::*;
use crate::utils::error::Error;
use crate::utils::helper::{diff_time, parse_bytes_n};
use crate::POOL;
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
    let conn = &POOL.clone().get().expect("Mysql pool connection error");
    loop {
        let claims_page: Vec<ClaimDb> = claimed_cota_nft_kv_pairs
            .select(get_selection())
            .filter(lock_hash_crc.eq(lock_hash_crc_))
            .filter(lock_hash.eq(lock_hash_hex.clone()))
            .limit(PAGE_SIZE)
            .offset(PAGE_SIZE * page)
            .load::<ClaimCotaNft>(conn)
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
