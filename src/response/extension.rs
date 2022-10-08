use crate::response::helper::Inserter;
use ckb_types::prelude::Entity;
use cota_smt::extension::ExtensionEntries;
use cota_smt::smt::H256;
use jsonrpc_http_server::jsonrpc_core::serde_json::Map;
use jsonrpc_http_server::jsonrpc_core::Value;

pub fn parse_extension_smt(
    (root_hash, extension_entries): (H256, ExtensionEntries),
    block_number: u64,
) -> Value {
    let extension_entry = hex::encode(extension_entries.as_slice());
    let define_root_hash = hex::encode(root_hash.as_slice());
    let mut map = Map::new();
    map.insert_str("smt_root_hash", define_root_hash);
    map.insert_str("extension_smt_entry", extension_entry);
    map.insert_u64("block_number", block_number);
    Value::Object(map)
}
