use crate::business::transaction::{CotaTransaction, HistoryTransaction};
use crate::response::helper::Inserter;
use crate::utils::error::Error;
use jsonrpc_http_server::jsonrpc_core::serde_json::Map;
use jsonrpc_http_server::jsonrpc_core::Value;

use super::helper::parse_json_err;

pub fn parse_history_transactions(
    txs: Vec<HistoryTransaction>,
    total: i64,
    page_size: i64,
    block_height: u64,
) -> Result<Value, Error> {
    let mut transactions: Vec<Value> = Vec::new();
    for tx in txs {
        transactions.push(parse_tx_value(tx)?);
    }
    let mut map = Map::new();
    map.insert_i64("total", total);
    map.insert_i64("page_size", page_size);
    map.insert_u64("block_number", block_height);
    map.insert_array("transactions", transactions);
    Ok(Value::Object(map))
}

fn parse_tx_value(tx: HistoryTransaction) -> Result<Value, Error> {
    let tx_json = serde_json::to_string(&tx).map_err(parse_json_err)?;
    let tx_map: Map<String, Value> = serde_json::from_str(&tx_json).map_err(parse_json_err)?;
    Ok(Value::Object(tx_map))
}

pub fn parse_cota_transactions(
    txs: Vec<CotaTransaction>,
    block_height: u64,
) -> Result<Value, Error> {
    let mut transactions: Vec<Value> = Vec::new();
    for tx in txs {
        transactions.push(parse_transaction(tx)?);
    }
    let mut map = Map::new();
    map.insert_u64("block_number", block_height);
    map.insert_array("transactions", transactions);
    Ok(Value::Object(map))
}

fn parse_transaction(tx: CotaTransaction) -> Result<Value, Error> {
    let tx_json = serde_json::to_string(&tx).map_err(parse_json_err)?;
    let tx_map: Map<String, Value> = serde_json::from_str(&tx_json).map_err(parse_json_err)?;
    Ok(Value::Object(tx_map))
}
