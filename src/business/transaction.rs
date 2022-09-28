use crate::business::helper::address_from_script;
use crate::ckb::rpc::get_block_timestamp;
use crate::models::withdrawal::transaction::{
    get_all_transactions, get_first_tx_block_number, get_transactions_by_block_number,
};
use crate::request::fetch::{FetchHistoryTxsReq, FetchTxsByBlockNumberReq};
use crate::utils::error::Error;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct HistoryTransaction {
    pub tx_hash:      String,
    pub block_number: u64,
    pub age:          u64,
    pub from:         String,
    pub to:           String,
    pub tx_type:      String,
}

pub async fn get_history_transactions(
    req: FetchHistoryTxsReq,
) -> Result<(Vec<HistoryTransaction>, i64, u64), Error> {
    let FetchHistoryTxsReq {
        cota_id,
        token_index,
        page,
        page_size,
    } = req;
    let first_tx_block_number = get_first_tx_block_number(cota_id, token_index)?;
    let mint_timestamp = get_block_timestamp(first_tx_block_number).await?;
    let (history_txs, total, block_height) =
        get_all_transactions(cota_id, token_index, page, page_size)?;
    let mut txs: Vec<HistoryTransaction> = Vec::with_capacity(history_txs.len());
    for tx in history_txs {
        let age = get_block_timestamp(tx.block_number).await?;
        let to = address_from_script(&tx.receiver_lock_script)?;
        let from = address_from_script(&tx.lock_script)?;
        let tx_type = if age == mint_timestamp {
            "mint".to_string()
        } else {
            "transfer".to_string()
        };
        txs.push(HistoryTransaction {
            tx_hash: format!("0x{:}", tx.tx_hash),
            block_number: tx.block_number,
            age,
            to,
            from,
            tx_type,
        });
    }
    Ok((txs, total, block_height))
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct CotaTransaction {
    pub cota_id:      String,
    pub token_index:  String,
    pub tx_hash:      String,
    pub block_number: u64,
    pub from:         String,
    pub to:           String,
    pub tx_type:      String,
}

pub async fn get_txs_by_block_number(
    req: FetchTxsByBlockNumberReq,
) -> Result<(Vec<CotaTransaction>, u64), Error> {
    let FetchTxsByBlockNumberReq { block_number } = req;
    let (history_txs, block_height) = get_transactions_by_block_number(block_number)?;
    let mut txs: Vec<CotaTransaction> = Vec::with_capacity(history_txs.len());
    for tx in history_txs {
        let first_tx_block_number = get_first_tx_block_number(tx.cota_id_bytes, tx.token_index)?;
        let to = address_from_script(&tx.receiver_lock_script)?;
        let from = address_from_script(&tx.lock_script)?;
        let tx_type = if block_number == first_tx_block_number {
            "mint".to_string()
        } else {
            "transfer".to_string()
        };
        txs.push(CotaTransaction {
            tx_hash: format!("0x{:}", tx.tx_hash),
            cota_id: format!("0x{:}", tx.cota_id),
            token_index: format!("0x{}", hex::encode(tx.token_index)),
            block_number,
            to,
            from,
            tx_type,
        });
    }
    Ok((txs, block_height))
}
