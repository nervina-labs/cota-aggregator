use crate::response::helper::Inserter;
use ckb_types::prelude::Entity;
use joyid_smt::joyid::SocialUnlockEntries;
use jsonrpc_http_server::jsonrpc_core::serde_json::Map;
use jsonrpc_http_server::jsonrpc_core::Value;

pub fn parse_social_unlock(unlock_entries: SocialUnlockEntries, block_number: u64) -> Value {
    let unlock_entry = hex::encode(unlock_entries.as_slice());
    let mut map = Map::new();
    map.insert_str("unlock_entry", unlock_entry);
    map.insert_u64("block_number", block_number);
    Value::Object(map)
}
