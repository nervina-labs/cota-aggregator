use crate::entries::sequential_transfer::SequentialTransferResult;
use crate::response::helper::Inserter;
use ckb_types::prelude::Entity;
use cota_smt::smt::H256;
use cota_smt::transfer::TransferCotaNFTV2Entries;
use cota_smt::transfer_update::TransferUpdateCotaNFTV2Entries;
use jsonrpc_http_server::jsonrpc_core::serde_json::Map;
use jsonrpc_http_server::jsonrpc_core::Value;

pub fn parse_transfer_smt(
    (root_hash, transfer_entries, block_hash): (H256, TransferCotaNFTV2Entries, H256),
    block_number: u64,
) -> Value {
    let transfer_entry = hex::encode(transfer_entries.as_slice());
    let transfer_root_hash = hex::encode(root_hash.as_slice());
    let withdraw_block_hash = hex::encode(block_hash.as_slice());
    let mut map = Map::new();
    map.insert_str("smt_root_hash", transfer_root_hash);
    map.insert_str("transfer_smt_entry", transfer_entry);
    map.insert_str("withdraw_block_hash", withdraw_block_hash);
    map.insert_u64("block_number", block_number);
    Value::Object(map)
}

pub fn parse_transfer_update_smt(
    (root_hash, transfer_update_entries, block_hash): (H256, TransferUpdateCotaNFTV2Entries, H256),
    block_number: u64,
) -> Value {
    let transfer_update_entry = hex::encode(transfer_update_entries.as_slice());
    let transfer_root_hash = hex::encode(root_hash.as_slice());
    let withdraw_block_hash = hex::encode(block_hash.as_slice());
    let mut map = Map::new();
    map.insert_str("smt_root_hash", transfer_root_hash);
    map.insert_str("transfer_update_smt_entry", transfer_update_entry);
    map.insert_str("withdraw_block_hash", withdraw_block_hash);
    map.insert_u64("block_number", block_number);
    Value::Object(map)
}

pub fn parse_sequential_transfer_smt(
    (root_hash, transfer_entries, current_subkey_entries, next_subkey_entries, block_hash): SequentialTransferResult,
    block_number: u64,
) -> Value {
    let transfer_entry = hex::encode(transfer_entries.as_slice());
    let transfer_root_hash = hex::encode(root_hash.as_slice());
    let withdraw_block_hash = hex::encode(block_hash.as_slice());
    let mut map = Map::new();
    map.insert_str("smt_root_hash", transfer_root_hash);
    map.insert_str("transfer_smt_entry", transfer_entry);
    map.insert_str("withdraw_block_hash", withdraw_block_hash);
    map.insert_u64("block_number", block_number);
    if let Some(subkey_unlock) = current_subkey_entries {
        map.insert_str("subkey_unlock_entry", hex::encode(subkey_unlock.as_slice()));
    }
    if let Some(subkey_unlock) = next_subkey_entries {
        map.insert_str(
            "next_subkey_unlock_entry",
            hex::encode(subkey_unlock.as_slice()),
        );
    }
    Value::Object(map)
}
