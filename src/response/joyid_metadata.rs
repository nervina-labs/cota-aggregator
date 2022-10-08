use crate::models::joyid::JoyIDInfo;
use crate::utils::error::Error;
use jsonrpc_http_server::jsonrpc_core::serde_json::Map;
use jsonrpc_http_server::jsonrpc_core::Value;

use super::helper::{parse_json_err, Inserter};

pub fn parse_joyid_metadata_response(
    joyid_info: Option<JoyIDInfo>,
    block_number: u64,
) -> Result<Value, Error> {
    let joyid = joyid_info.map_or(JoyIDInfo::default(), |info| info);
    let joyid_json = serde_json::to_string(&joyid).map_err(parse_json_err)?;
    let mut map: Map<String, Value> = serde_json::from_str(&joyid_json).map_err(parse_json_err)?;
    map.insert_u64("block_number", block_number);
    Ok(Value::Object(map))
}
