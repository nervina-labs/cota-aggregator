use super::helper::{
    establish_connection, parse_cota_id_and_token_index_pairs, parse_lock_hash, SqlConnection,
};
use crate::models::scripts::get_script_map_by_ids;
use crate::models::DBResult;
use crate::schema::withdraw_cota_nft_kv_pairs::dsl::withdraw_cota_nft_kv_pairs;
use crate::schema::withdraw_cota_nft_kv_pairs::*;
use crate::utils::error::Error;
use crate::utils::helper::parse_bytes_n;
use diesel::*;
use log::error;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Queryable, Debug)]
pub struct WithdrawCotaNft {
    pub cota_id:                 String,
    pub token_index:             u32,
    pub out_point:               String,
    pub state:                   u8,
    pub configure:               u8,
    pub characteristic:          String,
    pub receiver_lock_script_id: i64,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WithdrawDb {
    pub cota_id:              [u8; 20],
    pub token_index:          [u8; 4],
    pub out_point:            [u8; 24],
    pub state:                u8,
    pub configure:            u8,
    pub characteristic:       [u8; 20],
    pub receiver_lock_script: Vec<u8>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WithdrawNFTDb {
    pub cota_id:        [u8; 20],
    pub token_index:    [u8; 4],
    pub state:          u8,
    pub configure:      u8,
    pub characteristic: [u8; 20],
}

pub fn get_withdrawal_cota_by_lock_hash_with_conn(
    conn: &SqlConnection,
    lock_hash_: [u8; 32],
    cota_id_and_token_index_pairs: Option<Vec<([u8; 20], [u8; 4])>>,
) -> DBResult<WithdrawDb> {
    let (lock_hash_hex, lock_hash_crc_) = parse_lock_hash(lock_hash_);
    let select = withdraw_cota_nft_kv_pairs
        .select((
            cota_id,
            token_index,
            out_point,
            state,
            configure,
            characteristic,
            receiver_lock_script_id,
        ))
        .filter(lock_hash_crc.eq(lock_hash_crc_))
        .filter(lock_hash.eq(lock_hash_hex));
    let result = match cota_id_and_token_index_pairs {
        Some(pairs) => {
            let (cota_id_array, token_index_array) = parse_cota_id_and_token_index_pairs(pairs);
            select
                .filter(cota_id.eq_any(cota_id_array))
                .filter(token_index.eq_any(token_index_array))
                .load::<WithdrawCotaNft>(conn)
        }
        None => select.load::<WithdrawCotaNft>(conn),
    };
    let withdraw_cota_nfts: Vec<WithdrawCotaNft> = result.map_err(|e| {
        error!("Query withdraw error: {}", e.to_string());
        Error::DatabaseQueryError(e.to_string())
    })?;
    parse_withdraw_db(conn, withdraw_cota_nfts)
}

pub fn get_withdrawal_cota_by_lock_hash(
    lock_hash_: [u8; 32],
    cota_id_and_token_index_pairs: Option<Vec<([u8; 20], [u8; 4])>>,
) -> DBResult<WithdrawDb> {
    get_withdrawal_cota_by_lock_hash_with_conn(
        &establish_connection(),
        lock_hash_,
        cota_id_and_token_index_pairs,
    )
}

pub fn get_withdrawal_cota_by_cota_ids(
    conn: &SqlConnection,
    lock_hash_: [u8; 32],
    cota_ids: Vec<[u8; 20]>,
    page: i64,
    page_size: i64,
) -> DBResult<WithdrawDb> {
    let (lock_hash_hex, lock_hash_crc_) = parse_lock_hash(lock_hash_);
    let cota_ids_: Vec<String> = cota_ids
        .into_iter()
        .map(|cota_id_| hex::encode(&cota_id_))
        .collect();
    let withdraw_cota_nfts: Vec<WithdrawCotaNft> = withdraw_cota_nft_kv_pairs
        .select((
            cota_id,
            token_index,
            out_point,
            state,
            configure,
            characteristic,
            receiver_lock_script_id,
        ))
        .filter(lock_hash_crc.eq(lock_hash_crc_))
        .filter(lock_hash.eq(lock_hash_hex))
        .filter(cota_id.eq_any(cota_ids_))
        .limit(page_size)
        .offset(page_size * page)
        .load::<WithdrawCotaNft>(conn)
        .map_err(|e| {
            error!("Query withdraw error: {}", e.to_string());
            Error::DatabaseQueryError(e.to_string())
        })?;
    parse_withdraw_db(conn, withdraw_cota_nfts)
}

pub fn get_withdrawal_cota_by_script_id(
    conn: &SqlConnection,
    script_id: i64,
    page: i64,
    page_size: i64,
) -> DBResult<WithdrawNFTDb> {
    let withdraw_cota_nfts: Vec<WithdrawCotaNft> = withdraw_cota_nft_kv_pairs
        .select((
            cota_id,
            token_index,
            out_point,
            state,
            configure,
            characteristic,
            receiver_lock_script_id,
        ))
        .filter(receiver_lock_script_id.eq(script_id))
        .limit(page_size)
        .offset(page_size * page)
        .load::<WithdrawCotaNft>(conn)
        .map_err(|e| {
            error!("Query withdraw error: {}", e.to_string());
            Error::DatabaseQueryError(e.to_string())
        })?;
    parse_withdraw_cota_nft(withdraw_cota_nfts)
}

fn parse_withdraw_db(
    conn: &SqlConnection,
    withdrawals: Vec<WithdrawCotaNft>,
) -> DBResult<WithdrawDb> {
    if withdrawals.is_empty() {
        return Ok(vec![]);
    }
    let receiver_lock_script_ids: Vec<i64> = withdrawals
        .iter()
        .map(|withdrawal| withdrawal.receiver_lock_script_id)
        .collect();
    let mut withdraw_db_vec: Vec<WithdrawDb> = vec![];
    let script_map = get_script_map_by_ids(conn, receiver_lock_script_ids)?;
    for withdrawal in withdrawals {
        let lock_script = script_map
            .get(&withdrawal.receiver_lock_script_id)
            .ok_or(Error::DatabaseQueryError("scripts".to_owned()))?
            .clone();
        withdraw_db_vec.push(WithdrawDb {
            cota_id:              parse_bytes_n::<20>(withdrawal.cota_id).unwrap(),
            token_index:          withdrawal.token_index.to_be_bytes(),
            configure:            withdrawal.configure,
            state:                withdrawal.state,
            characteristic:       parse_bytes_n::<20>(withdrawal.characteristic).unwrap(),
            receiver_lock_script: lock_script,
            out_point:            parse_bytes_n::<24>(withdrawal.out_point).unwrap(),
        })
    }
    Ok(withdraw_db_vec)
}

fn parse_withdraw_cota_nft(withdrawals: Vec<WithdrawCotaNft>) -> DBResult<WithdrawNFTDb> {
    if withdrawals.is_empty() {
        return Ok(vec![]);
    }
    let withdraw_db_vec: Vec<WithdrawNFTDb> = withdrawals
        .into_iter()
        .map(|withdrawal| WithdrawNFTDb {
            cota_id:        parse_bytes_n::<20>(withdrawal.cota_id).unwrap(),
            token_index:    withdrawal.token_index.to_be_bytes(),
            configure:      withdrawal.configure,
            state:          withdrawal.state,
            characteristic: parse_bytes_n::<20>(withdrawal.characteristic).unwrap(),
        })
        .collect();
    Ok(withdraw_db_vec)
}
