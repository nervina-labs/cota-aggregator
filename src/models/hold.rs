use super::helper::{establish_connection, parse_cota_id_and_token_index_pairs, parse_lock_hash};
use crate::models::block::get_syncer_tip_block_number_with_conn;
use crate::models::helper::{SqlConnection, PAGE_SIZE};
use crate::models::{DBResult, DBTotalResult};
use crate::schema::hold_cota_nft_kv_pairs::dsl::hold_cota_nft_kv_pairs;
use crate::schema::hold_cota_nft_kv_pairs::*;
use crate::utils::error::Error;
use crate::utils::helper::parse_bytes_n;
use diesel::*;
use log::error;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Queryable, Debug)]
struct HoldCotaNft {
    pub cota_id:        String,
    pub token_index:    u32,
    pub state:          u8,
    pub configure:      u8,
    pub characteristic: String,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct HoldDb {
    pub cota_id:        [u8; 20],
    pub token_index:    [u8; 4],
    pub state:          u8,
    pub configure:      u8,
    pub characteristic: [u8; 20],
}

pub fn get_hold_cota_by_lock_hash_with_conn(
    conn: &SqlConnection,
    lock_hash_: [u8; 32],
    cota_id_and_token_index_pairs: Option<Vec<([u8; 20], [u8; 4])>>,
) -> DBResult<HoldDb> {
    let (lock_hash_hex, lock_hash_crc_) = parse_lock_hash(lock_hash_);
    let mut hold_vec: Vec<HoldDb> = vec![];
    let holds: Vec<HoldDb> = match cota_id_and_token_index_pairs {
        Some(pairs) => {
            let pair_vec = parse_cota_id_and_token_index_pairs(pairs);
            for (cota_id_str, token_index_u32) in pair_vec.into_iter() {
                let holds_: Vec<HoldDb> = hold_cota_nft_kv_pairs
                    .select(get_selection())
                    .filter(lock_hash_crc.eq(lock_hash_crc_))
                    .filter(lock_hash.eq(lock_hash_hex.clone()))
                    .filter(cota_id.eq(cota_id_str))
                    .filter(token_index.eq(token_index_u32))
                    .order(updated_at.desc())
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
        None => {
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
    };
    let block_height = get_syncer_tip_block_number_with_conn(conn)?;
    Ok((holds, block_height))
}

pub fn get_hold_cota_by_lock_hash(
    lock_hash_: [u8; 32],
    cota_id_and_token_index_pairs: Option<Vec<([u8; 20], [u8; 4])>>,
) -> DBResult<HoldDb> {
    get_hold_cota_by_lock_hash_with_conn(
        &establish_connection(),
        lock_hash_,
        cota_id_and_token_index_pairs,
    )
}

pub fn get_hold_cota_by_lock_hash_and_page(
    lock_hash_: [u8; 32],
    page: i64,
    page_size: i64,
) -> DBTotalResult<HoldDb> {
    let conn = &establish_connection();
    let (lock_hash_hex, lock_hash_crc_) = parse_lock_hash(lock_hash_);
    let total: i64 = hold_cota_nft_kv_pairs
        .filter(lock_hash_crc.eq(lock_hash_crc_))
        .filter(lock_hash.eq(lock_hash_hex.clone()))
        .count()
        .get_result::<i64>(conn)
        .map_err(|e| {
            error!("Query hold error: {}", e.to_string());
            Error::DatabaseQueryError(e.to_string())
        })?;
    let block_height: u64 = get_syncer_tip_block_number_with_conn(conn)?;

    let holds: Vec<HoldDb> = hold_cota_nft_kv_pairs
        .select(get_selection())
        .filter(lock_hash_crc.eq(lock_hash_crc_))
        .filter(lock_hash.eq(lock_hash_hex))
        .order(updated_at.desc())
        .limit(page_size)
        .offset(page_size * page)
        .load::<HoldCotaNft>(conn)
        .map_or_else(
            |e| {
                error!("Query hold error: {}", e.to_string());
                Err(Error::DatabaseQueryError(e.to_string()))
            },
            |holds| Ok(parse_hold_cota_nfts(holds)),
        )?;
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
