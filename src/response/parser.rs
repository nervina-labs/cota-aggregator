use crate::models::hold::HoldDb;
use crate::models::withdrawal::WithdrawDb;
use jsonrpc_http_server::jsonrpc_core::serde_json::Map;
use jsonrpc_http_server::jsonrpc_core::{Error, Value};

pub fn parse_hold_response(holds: Vec<HoldDb>) -> Result<Map<String, Value>, Error> {
    Ok(Map::new())
}

pub fn parse_withdrawal_response(
    withdrawals: Vec<WithdrawDb>,
) -> Result<Map<String, Value>, Error> {
    Ok(Map::new())
}
