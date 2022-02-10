use super::helper::parse_lock_hash;
use crate::models::helper::SqlConnection;
use crate::models::DBResult;
use crate::schema::claimed_cota_nft_kv_pairs::dsl::claimed_cota_nft_kv_pairs;
use crate::schema::claimed_cota_nft_kv_pairs::*;
use crate::utils::error::Error;
use crate::utils::helper::parse_bytes_n;
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

pub fn get_claim_cota_by_lock_hash_with_conn(
    conn: &SqlConnection,
    lock_hash_: [u8; 32],
) -> DBResult<ClaimDb> {
    let (lock_hash_hex, lock_hash_crc_) = parse_lock_hash(lock_hash_);
    claimed_cota_nft_kv_pairs
        .select((cota_id, token_index, out_point))
        .filter(lock_hash_crc.eq(lock_hash_crc_))
        .filter(lock_hash.eq(lock_hash_hex))
        .load::<ClaimCotaNft>(conn)
        .map_or_else(
            |e| {
                error!("Query claim error: {}", e.to_string());
                Err(Error::DatabaseQueryError(e.to_string()))
            },
            |claims| Ok(parse_claimed_cota_nft(claims)),
        )
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
