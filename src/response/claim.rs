use crate::response::helper::Inserter;
use ckb_types::prelude::Entity;
use cota_smt::smt::H256;
use cota_smt::transfer::ClaimCotaNFTV2Entries;
use cota_smt::transfer_update::ClaimUpdateCotaNFTV2Entries;
use jsonrpc_http_server::jsonrpc_core::serde_json::Map;
use jsonrpc_http_server::jsonrpc_core::Value;

pub fn parse_claimed_response(claimed: bool, block_number: u64) -> Map<String, Value> {
    let mut map = Map::new();
    map.insert("claimed".to_string(), Value::Bool(claimed));
    map.insert_u64("block_number", block_number);
    map
}

pub fn parse_claimed_smt(
    (root_hash, claim_entries, block_hash): (H256, ClaimCotaNFTV2Entries, H256),
    block_number: u64,
) -> Map<String, Value> {
    let claim_entry = hex::encode(claim_entries.as_slice());
    let claim_root_hash = hex::encode(root_hash.as_slice());
    let withdraw_block_hash = hex::encode(block_hash.as_slice());
    let mut map = Map::new();
    map.insert_str("smt_root_hash", claim_root_hash);
    map.insert_str("claim_smt_entry", claim_entry);
    map.insert_str("withdraw_block_hash", withdraw_block_hash);
    map.insert_u64("block_number", block_number);
    map
}

pub fn parse_claimed_update_smt(
    (root_hash, claim_update_entries, block_hash): (H256, ClaimUpdateCotaNFTV2Entries, H256),
    block_number: u64,
) -> Map<String, Value> {
    let claim_update_entry = hex::encode(claim_update_entries.as_slice());
    let claim_root_hash = hex::encode(root_hash.as_slice());
    let withdraw_block_hash = hex::encode(block_hash.as_slice());
    let mut map = Map::new();
    map.insert_str("smt_root_hash", claim_root_hash);
    map.insert_str("claim_update_smt_entry", claim_update_entry);
    map.insert_str("withdraw_block_hash", withdraw_block_hash);
    map.insert_u64("block_number", block_number);
    map
}
