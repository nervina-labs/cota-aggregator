use crate::models::class::ClassInfoDb;
use crate::models::define::DefineDb;
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
    define_info: Option<DefineDb>,
    class_info: Option<ClassInfoDb>,
    block_number: u64,
) -> Map<String, Value> {
    let mut map = Map::new();
    match define_info {
        Some(define) => {
            map.insert_u32("total", define.total);
            map.insert_u32("issued", define.issued);
            map.insert_str(
                "configure",
                format!("0x{}", hex::encode(&[define.configure])),
            );
        }
        None => {
            map.insert_null("total");
            map.insert_null("issued");
            map.insert_null("configure");
        }
    }
    match class_info {
        Some(class) => {
            map.insert_str("name", class.name);
            map.insert_str("description", class.description);
            map.insert_str("image", class.image);
            map.insert_str("audio", class.audio);
            map.insert_str("video", class.video);
            map.insert_str("model", class.model);
            map.insert_str("meta_characteristic", class.characteristic);
            map.insert_str("properties", class.properties);
        }
        None => {
            map.insert_null("name");
            map.insert_null("description");
            map.insert_null("image");
            map.insert_null("audio");
            map.insert_null("video");
            map.insert_null("model");
            map.insert_null("meta_characteristic");
            map.insert_null("properties");
        }
    }
    map.insert_u64("block_number", block_number);
    map
}
