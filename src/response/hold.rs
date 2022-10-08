use crate::models::class::ClassInfoDb;
use crate::models::hold::HoldDb;
use crate::response::helper::Inserter;
use crate::utils::error::Error;
use jsonrpc_http_server::jsonrpc_core::serde_json::Map;
use jsonrpc_http_server::jsonrpc_core::Value;

use super::helper::parse_json_err;

pub fn parse_hold_response(
    holds: Vec<(HoldDb, Option<ClassInfoDb>)>,
    total: i64,
    page_size: i64,
    block_number: u64,
) -> Result<Value, Error> {
    let mut nfts: Vec<Value> = Vec::new();
    for hold in holds {
        nfts.push(parse_hold_value(hold)?);
    }
    let mut map = Map::new();
    map.insert_i64("total", total);
    map.insert_i64("page_size", page_size);
    map.insert_u64("block_number", block_number);
    map.insert_array("nfts", nfts);
    Ok(Value::Object(map))
}

fn parse_hold_value((hold, class_info): (HoldDb, Option<ClassInfoDb>)) -> Result<Value, Error> {
    let mut map = Map::new();
    map.insert_hex("cota_id", &hold.cota_id);
    map.insert_hex("token_index", &hold.token_index);
    map.insert_hex("state", &[hold.state]);
    map.insert_hex("configure", &[hold.configure]);
    map.insert_hex("characteristic", &hold.characteristic);

    let class = class_info.map_or(ClassInfoDb::default(), |class| class);
    let class_json = serde_json::to_string(&class).map_err(parse_json_err)?;
    let mut class_map: Map<String, Value> =
        serde_json::from_str(&class_json).map_err(parse_json_err)?;
    map.append(&mut class_map);
    Ok(Value::Object(map))
}

pub fn parse_owned_nft_count(count: i64, block_number: u64) -> Value {
    let mut map = Map::new();
    map.insert_i64("count", count);
    map.insert_u64("block_number", block_number);
    Value::Object(map)
}
