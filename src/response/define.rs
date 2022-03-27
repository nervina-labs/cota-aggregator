use crate::response::helper::Inserter;
use ckb_types::prelude::Entity;
use cota_smt::define::DefineCotaNFTEntries;
use cota_smt::smt::H256;
use jsonrpc_http_server::jsonrpc_core::serde_json::Map;
use jsonrpc_http_server::jsonrpc_core::Value;

pub fn parse_define_smt(
    (root_hash, define_entries): (H256, DefineCotaNFTEntries),
    block_number: u64,
) -> Map<String, Value> {
    let define_entry = hex::encode(define_entries.as_slice());
    let define_root_hash = hex::encode(root_hash.as_slice());
    let mut map = Map::new();
    map.insert_str("smt_root_hash", define_root_hash);
    map.insert_str("define_smt_entry", define_entry);
    map.insert_u64("block_number", block_number);
    map
}

pub fn parse_define_info(
    define_info: Option<(u32, u32, u8)>,
    block_number: u64,
) -> Map<String, Value> {
    let mut map = Map::new();
    match define_info {
        Some(define) => {
            map.insert_u32("total", define.0);
            map.insert_u32("issued", define.1);
            map.insert_str("configure", format!("0x{}", hex::encode(&[define.2])));
        }
        None => {
            map.insert_null("total");
            map.insert_null("issued");
            map.insert_null("configure");
        }
    }
    map.insert_u64("block_number", block_number);
    map
}
