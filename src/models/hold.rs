use super::helper::{establish_connection, parse_cota_id_and_token_index_pairs, parse_lock_hash};
use crate::models::helper::SqlConnection;
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
    let select = hold_cota_nft_kv_pairs
        .select((cota_id, token_index, configure, state, characteristic))
        .filter(lock_hash_crc.eq(lock_hash_crc_))
        .filter(lock_hash.eq(lock_hash_hex));
    let result = match cota_id_and_token_index_pairs {
        Some(pairs) => {
            let (cota_id_array, token_index_array) = parse_cota_id_and_token_index_pairs(pairs);
            select
                .filter(cota_id.eq_any(cota_id_array))
                .filter(token_index.eq_any(token_index_array))
                .load::<HoldCotaNft>(conn)
        }
        None => select.load::<HoldCotaNft>(conn),
    };
    result.map_or_else(
        |e| {
            error!("Query hold error: {}", e.to_string());
            Err(Error::DatabaseQueryError(e.to_string()))
        },
        |holds| Ok(parse_hold_cota_nft(holds)),
    )
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

    let holds: Vec<HoldDb> = hold_cota_nft_kv_pairs
        .select((cota_id, token_index, configure, state, characteristic))
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
            |holds| Ok(parse_hold_cota_nft(holds)),
        )?;
    Ok((holds, total))
}

fn parse_hold_cota_nft(holds: Vec<HoldCotaNft>) -> Vec<HoldDb> {
    holds
        .into_iter()
        .map(|hold| HoldDb {
            cota_id:        parse_bytes_n::<20>(hold.cota_id).unwrap(),
            token_index:    hold.token_index.to_be_bytes(),
            state:          hold.state,
            configure:      hold.configure,
            characteristic: parse_bytes_n::<20>(hold.characteristic).unwrap(),
        })
        .collect()
}
