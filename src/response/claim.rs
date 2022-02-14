use crate::response::helper::Inserter;
use jsonrpc_http_server::jsonrpc_core::serde_json::Map;
use jsonrpc_http_server::jsonrpc_core::Value;

pub fn parse_claimed_response(claimed: bool, block_number: u64) -> Map<String, Value> {
    let mut map = Map::new();
    map.insert("claimed".to_string(), Value::Bool(claimed));
    map.insert_u64("block_number", block_number);
    map
}

pub fn parse_claimed_smt(
    (root_hash, smt_entry): (String, String),
    block_number: u64,
) -> Map<String, Value> {
    let mut map = Map::new();
    map.insert_str("smt_root_hash", root_hash);
    map.insert_str("claim_smt_entry", smt_entry);
    map.insert_u64("block_number", block_number);
    map
}
