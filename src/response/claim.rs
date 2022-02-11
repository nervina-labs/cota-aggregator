use jsonrpc_http_server::jsonrpc_core::serde_json::Map;
use jsonrpc_http_server::jsonrpc_core::Value;

pub fn parse_claimed_response(claimed: bool) -> Map<String, Value> {
    let mut map = Map::new();
    map.insert("claimed".to_string(), Value::Bool(claimed));
    map
}
