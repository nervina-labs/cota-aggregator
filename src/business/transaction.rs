use crate::ckb::rpc::get_block_timestamp;
use crate::models::withdrawal::{get_all_transactions, get_first_transaction};
use crate::request::fetch::FetchHistoryTxsReq;
use crate::utils::error::Error;
use ckb_sdk::{Address, AddressPayload, NetworkType};
use ckb_types::packed::Script;
use molecule::prelude::Entity;
use serde_json::from_str;
use std::env;

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
        let is_mainnet: bool = match env::var("IS_MAINNET") {
            Ok(mainnet) => from_str::<bool>(&mainnet).unwrap(),
            Err(_e) => false,
        };
        let mint_timestamp = get_block_timestamp(first_tx.block_number).await?;
        let (history_txs, total, block_height) =
            get_all_transactions(cota_id, token_index, page, page_size)?;
        let mut txs: Vec<HistoryTransaction> = Vec::with_capacity(history_txs.len());
        for tx in history_txs {
            let timestamp = get_block_timestamp(tx.block_number).await?;
            println!("timestamp1: {:?}, 2: {:?}", mint_timestamp, timestamp);
            let age = timestamp - mint_timestamp;
            let to = address_from_script(&tx.receiver_lock_script, is_mainnet)?;
            let from = address_from_script(&tx.lock_script, is_mainnet)?;
            let tx_type = if timestamp == mint_timestamp {
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

fn address_from_script(slice: &[u8], is_mainnet: bool) -> Result<String, Error> {
    let payload =
        AddressPayload::from(Script::from_slice(slice).map_err(|_| Error::CKBScriptError)?);
    println!("payload: {:?}", payload);
    let network = if is_mainnet {
        NetworkType::Mainnet
    } else {
        NetworkType::Testnet
    };
    Ok(Address::new(network, payload, true).to_string())
}
