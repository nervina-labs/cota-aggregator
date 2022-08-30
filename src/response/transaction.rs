use crate::business::transaction::{CotaTransaction, HistoryTransaction};
use crate::response::helper::Inserter;
use jsonrpc_http_server::jsonrpc_core::serde_json::Map;
use jsonrpc_http_server::jsonrpc_core::Value;

pub fn parse_history_transactions(
    txs: Vec<HistoryTransaction>,
    total: i64,
    page_size: i64,
    block_height: u64,
) -> Map<String, Value> {
    let transactions = txs.into_iter().map(parse_mint_value).collect();
    let mut map = Map::new();
    map.insert_i64("total", total);
    map.insert_i64("page_size", page_size);
    map.insert_u64("block_number", block_height);
    map.insert_array("transactions", transactions);
    map
}

fn parse_mint_value(tx: HistoryTransaction) -> Value {
    let mut map = Map::new();
    map.insert_str("tx_hash", tx.tx_hash);
    map.insert_u64("block_number", tx.block_number);
    map.insert_u64("age", tx.age);
    map.insert_str("from", tx.from);
    map.insert_str("to", tx.to);
    map.insert_str("type", tx.tx_type);
    Value::Object(map)
}

pub fn parse_cota_transactions(txs: Vec<CotaTransaction>, block_height: u64) -> Map<String, Value> {
    let transactions = txs.into_iter().map(parse_transaction).collect();
    let mut map = Map::new();
    map.insert_u64("block_number", block_height);
    map.insert_array("transactions", transactions);
    map
}

fn parse_transaction(tx: CotaTransaction) -> Value {
    let mut map = Map::new();
    map.insert_str("tx_hash", tx.tx_hash);
    map.insert_u64("block_number", tx.block_number);
    map.insert_str("from", tx.from);
    map.insert_str("to", tx.to);
    map.insert_str("type", tx.tx_type);
    map.insert_str("cota_id", tx.cota_id);
    map.insert_str("token_index", tx.token_index);
    Value::Object(map)
}
