use crate::business::helper::address_from_script;
use crate::ckb::rpc::get_block_timestamp;
use crate::models::withdrawal::{get_all_transactions, get_first_transaction};
use crate::request::fetch::FetchHistoryTxsReq;
use crate::utils::error::Error;

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
    let first_tx_opt = get_first_transaction(cota_id, token_index)?;
    return if let Some(first_tx) = first_tx_opt {
        let mint_timestamp = get_block_timestamp(first_tx.block_number).await?;
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
    } else {
        Err(Error::CotaIdAndTokenIndexHasNoTxs)
    };
}
