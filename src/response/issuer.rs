use crate::models::issuer::IssuerInfoDb;
use crate::response::helper::Inserter;
use jsonrpc_http_server::jsonrpc_core::serde_json::Map;
use jsonrpc_http_server::jsonrpc_core::Value;

pub fn parse_issuer_response(
    issuer_info: Option<IssuerInfoDb>,
    block_number: u64,
) -> Map<String, Value> {
    let mut map = Map::new();
    match issuer_info {
        Some(issuer) => {
            map.insert_str("name", issuer.name);
            map.insert_str("avatar", issuer.avatar);
            map.insert_str("description", issuer.description);
        }
        None => {
            map.insert_null("name");
            map.insert_null("avatar");
            map.insert_null("description");
        }
    }
    map.insert_u64("block_number", block_number);
    map
}
