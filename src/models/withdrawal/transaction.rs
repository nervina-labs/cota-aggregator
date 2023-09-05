use crate::models::block::get_syncer_tip_block_number;
use crate::models::helper::generate_crc;
use crate::models::scripts::get_script_map_by_ids;
use crate::models::{get_conn, DBResult, DBTotalResult};
use crate::schema::withdraw_cota_nft_kv_pairs::dsl::withdraw_cota_nft_kv_pairs;
use crate::schema::withdraw_cota_nft_kv_pairs::*;
use crate::utils::error::Error;
use crate::utils::helper::parse_bytes_n;
use diesel::*;
use log::error;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Serialize, Deserialize, Queryable, Debug, Clone)]
pub struct WithdrawTxDb {
    pub block_number:            u64,
    pub tx_hash:                 String,
    pub lock_script_id:          i64,
    pub receiver_lock_script_id: i64,
}

#[derive(Debug, Clone)]
pub struct WithdrawHistoryTx {
    pub block_number:         u64,
    pub tx_hash:              String,
    pub lock_script:          Vec<u8>,
    pub receiver_lock_script: Vec<u8>,
}

pub fn get_all_transactions(
    cota_id_: [u8; 20],
    token_index_: [u8; 4],
    page: i64,
    page_size: i64,
) -> DBTotalResult<WithdrawHistoryTx> {
    let conn = &get_conn();
    let cota_id_hex = hex::encode(cota_id_);
    let token_index_u32 = u32::from_be_bytes(token_index_);
    let cota_id_crc_u32 = generate_crc(cota_id_hex.as_bytes());
    let total = withdraw_cota_nft_kv_pairs
        .filter(cota_id_crc.eq(cota_id_crc_u32))
        .filter(token_index.eq(token_index_u32))
        .filter(cota_id.eq(cota_id_hex.clone()))
        .count()
        .get_result::<i64>(conn)
        .map_err(|e| {
            error!("Query withdraw error: {}", e.to_string());
            Error::DatabaseQueryInvalid(e.to_string())
        })?;

    let db_history_txs: Vec<WithdrawTxDb> = withdraw_cota_nft_kv_pairs
        .select((
            block_number,
            tx_hash,
            lock_script_id,
            receiver_lock_script_id,
        ))
        .filter(cota_id_crc.eq(cota_id_crc_u32))
        .filter(token_index.eq(token_index_u32))
        .filter(cota_id.eq(cota_id_hex))
        .order(updated_at.desc())
        .limit(page_size)
        .offset(page_size * page)
        .load::<WithdrawTxDb>(conn)
        .map_err(|e| {
            error!(
                "Query withdraw history transactions error: {}",
                e.to_string()
            );
            Error::DatabaseQueryInvalid(e.to_string())
        })?;

    let mut script_id_set = BTreeSet::<i64>::new();
    for tx in db_history_txs.iter() {
        script_id_set.insert(tx.receiver_lock_script_id);
        script_id_set.insert(tx.lock_script_id);
    }
    let script_ids: Vec<i64> = script_id_set.into_iter().collect();
    let script_map = get_script_map_by_ids(script_ids)?;
    let history_transactions: Vec<WithdrawHistoryTx> = db_history_txs
        .into_iter()
        .map(|tx| WithdrawHistoryTx {
            tx_hash:              tx.tx_hash,
            block_number:         tx.block_number,
            lock_script:          script_map.get(&tx.lock_script_id).cloned().unwrap(),
            receiver_lock_script: script_map
                .get(&tx.receiver_lock_script_id)
                .cloned()
                .unwrap(),
        })
        .collect();
    let block_height = get_syncer_tip_block_number()?;
    Ok((history_transactions, total, block_height))
}

pub fn get_first_tx_block_number(cota_id_: [u8; 20], token_index_: [u8; 4]) -> Result<u64, Error> {
    let cota_id_hex = hex::encode(cota_id_);
    let token_index_u32 = u32::from_be_bytes(token_index_);
    let cota_id_crc_u32 = generate_crc(cota_id_hex.as_bytes());

    withdraw_cota_nft_kv_pairs
        .select(block_number)
        .filter(cota_id_crc.eq(cota_id_crc_u32))
        .filter(token_index.eq(token_index_u32))
        .filter(cota_id.eq(cota_id_hex))
        .order(block_number.asc())
        .first::<u64>(&get_conn())
        .map_err(|e| {
            error!("Query withdraw tx block number error: {}", e.to_string());
            Error::DatabaseQueryInvalid(e.to_string())
        })
}

#[derive(Serialize, Deserialize, Queryable, Debug, Clone)]
pub struct WithdrawTransactionDb {
    pub cota_id:                 String,
    pub token_index:             u32,
    pub tx_hash:                 String,
    pub lock_script_id:          i64,
    pub receiver_lock_script_id: i64,
}

#[derive(Serialize, Deserialize, Queryable, Debug, Clone)]
pub struct WithdrawTransaction {
    pub cota_id:              String,
    pub token_index:          [u8; 4],
    pub cota_id_bytes:        [u8; 20],
    pub tx_hash:              String,
    pub lock_script:          Vec<u8>,
    pub receiver_lock_script: Vec<u8>,
}
pub fn get_transactions_by_block_number(block_number_: u64) -> DBResult<WithdrawTransaction> {
    let db_transactions: Vec<WithdrawTransactionDb> = withdraw_cota_nft_kv_pairs
        .select((
            cota_id,
            token_index,
            tx_hash,
            lock_script_id,
            receiver_lock_script_id,
        ))
        .filter(block_number.eq(block_number_))
        .load::<WithdrawTransactionDb>(&get_conn())
        .map_err(|e| {
            error!("Query withdraw transaction error: {}", e.to_string());
            Error::DatabaseQueryInvalid(e.to_string())
        })?;

    let mut script_id_set = BTreeSet::<i64>::new();
    for tx in db_transactions.iter() {
        script_id_set.insert(tx.receiver_lock_script_id);
        script_id_set.insert(tx.lock_script_id);
    }
    let script_ids: Vec<i64> = script_id_set.into_iter().collect();
    let script_map = get_script_map_by_ids(script_ids)?;
    let transactions: Vec<WithdrawTransaction> = db_transactions
        .into_iter()
        .map(|tx| WithdrawTransaction {
            cota_id:              tx.cota_id.clone(),
            token_index:          tx.token_index.to_be_bytes(),
            cota_id_bytes:        parse_bytes_n::<20>(tx.cota_id).unwrap(),
            tx_hash:              tx.tx_hash,
            lock_script:          script_map.get(&tx.lock_script_id).cloned().unwrap(),
            receiver_lock_script: script_map
                .get(&tx.receiver_lock_script_id)
                .cloned()
                .unwrap(),
        })
        .collect();
    let block_height = get_syncer_tip_block_number()?;
    Ok((transactions, block_height))
}
