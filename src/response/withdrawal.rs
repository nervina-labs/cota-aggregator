use crate::models::withdrawal::WithdrawNFTDb;
use crate::response::helper::Inserter;
use jsonrpc_http_server::jsonrpc_core::serde_json::Map;
use jsonrpc_http_server::jsonrpc_core::Value;

pub fn parse_withdrawal_response(
    withdrawals: Vec<WithdrawNFTDb>,
    total: i64,
    page_size: i64,
) -> Map<String, Value> {
    let nfts: Vec<Value> = withdrawals
        .into_iter()
        .map(parse_withdrawal_value)
        .collect();
    let mut map = Map::new();
    map.insert_i64("total", total);
    map.insert_i64("page_size", page_size);
    map.insert_array("nfts", nfts);
    map
}

fn parse_withdrawal_value(withdrawal: WithdrawNFTDb) -> Value {
    let mut map = Map::new();
    map.insert_hex("cota_id", &withdrawal.cota_id);
    map.insert_hex("token_index", &withdrawal.token_index);
    map.insert_hex("state", &[withdrawal.state]);
    map.insert_hex("configure", &[withdrawal.configure]);
    map.insert_hex("characteristic", &withdrawal.characteristic);
    Value::Object(map)
}
