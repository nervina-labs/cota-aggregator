use crate::response::helper::Inserter;
use crate::utils::error::Error;
use jsonrpc_http_server::jsonrpc_core::serde_json::Map;
use jsonrpc_http_server::jsonrpc_core::Value;

pub fn parse_ccid_response(
    ccid_info: Option<(String, u64)>,
    nickname_opt: Option<String>,
    block_number: u64,
) -> Result<Value, Error> {
    let mut map = Map::new();
    match ccid_info {
        Some(info) => {
            map.insert_str("address", info.0);
            map.insert_u64("ccid", info.1);
        }
        None => {
            map.insert_null("address");
            map.insert_null("ccid");
        }
    }
    match nickname_opt {
        Some(nickname) => {
            map.insert_str("joyid", nickname);
        }
        None => {
            map.insert_null("joyid");
        }
    }
    map.insert_u64("block_number", block_number);
    Ok(Value::Object(map))
}
