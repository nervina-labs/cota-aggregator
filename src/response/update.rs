use super::helper::Inserter;
use ckb_types::prelude::Entity;
use cota_smt::smt::H256;
use cota_smt::update::UpdateCotaNFTEntries;
use jsonrpc_http_server::jsonrpc_core::serde_json::Map;
use jsonrpc_http_server::jsonrpc_core::Value;

pub fn parse_update_smt(
    (root_hash, update_entries): (H256, UpdateCotaNFTEntries),
    block_number: u64,
) -> Map<String, Value> {
    let update_entry = hex::encode(update_entries.as_slice());
    let update_root_hash = hex::encode(root_hash.as_slice());
    let mut map = Map::new();
    map.insert_str("smt_root_hash", update_root_hash);
    map.insert_str("update_smt_entry", update_entry);
    map.insert_u64("block_number", block_number);
    map
}
