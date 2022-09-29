use crate::models::class::ClassInfoDb;
use crate::models::define::DefineDb;
use crate::response::helper::Inserter;
use crate::utils::error::Error;
use ckb_types::prelude::Entity;
use cota_smt::define::DefineCotaNFTEntries;
use cota_smt::smt::H256;
use jsonrpc_http_server::jsonrpc_core::serde_json::Map;
use jsonrpc_http_server::jsonrpc_core::Value;

use super::helper::parse_json_err;

pub fn parse_define_smt(
    (root_hash, define_entries): (H256, DefineCotaNFTEntries),
    block_number: u64,
) -> Value {
    let define_entry = hex::encode(define_entries.as_slice());
    let define_root_hash = hex::encode(root_hash.as_slice());
    let mut map = Map::new();
    map.insert_str("smt_root_hash", define_root_hash);
    map.insert_str("define_smt_entry", define_entry);
    map.insert_u64("block_number", block_number);
    Value::Object(map)
}

pub fn parse_define_info(
    define_info: Option<DefineDb>,
    class_info: Option<ClassInfoDb>,
    block_number: u64,
) -> Result<Value, Error> {
    let define = define_info.map_or(DefineDb::default(), |define| define);
    let define_json = serde_json::to_string(&define).map_err(parse_json_err)?;
    let mut map: Map<String, Value> = serde_json::from_str(&define_json).map_err(parse_json_err)?;

    let class = class_info.map_or(ClassInfoDb::default(), |class| class);
    let class_json = serde_json::to_string(&class).map_err(parse_json_err)?;
    let mut class_map: Map<String, Value> =
        serde_json::from_str(&class_json).map_err(parse_json_err)?;

    map.append(&mut class_map);
    map.insert_u64("block_number", block_number);
    Ok(Value::Object(map))
}
