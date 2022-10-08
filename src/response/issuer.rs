use crate::models::issuer::IssuerInfoDb;
use crate::response::helper::Inserter;
use crate::utils::error::Error;
use jsonrpc_http_server::jsonrpc_core::serde_json::Map;
use jsonrpc_http_server::jsonrpc_core::Value;

use super::helper::parse_json_err;

pub fn parse_issuer_response(
    issuer_info: Option<IssuerInfoDb>,
    block_number: u64,
) -> Result<Value, Error> {
    let issuer = issuer_info.map_or(IssuerInfoDb::default(), |issuer| issuer);
    let issuer_json = serde_json::to_string(&issuer).map_err(parse_json_err)?;
    let mut map: Map<String, Value> = serde_json::from_str(&issuer_json).map_err(parse_json_err)?;
    map.insert_u64("block_number", block_number);
    Ok(Value::Object(map))
}

pub fn parse_issuer_info_response(
    lock_hash: [u8; 32],
    issuer_info: Option<IssuerInfoDb>,
    block_number: u64,
) -> Result<Value, Error> {
    let issuer = issuer_info.map_or(IssuerInfoDb::default(), |issuer| issuer);
    let issuer_json = serde_json::to_string(&issuer).map_err(parse_json_err)?;
    let mut map: Map<String, Value> = serde_json::from_str(&issuer_json).map_err(parse_json_err)?;
    map.insert_u64("block_number", block_number);
    map.insert_hex("lock_hash", &lock_hash);
    Ok(Value::Object(map))
}
