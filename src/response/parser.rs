use crate::models::hold::HoldDb;
use crate::models::withdrawal::{WithdrawDb, WithdrawNFTDb};
use jsonrpc_http_server::jsonrpc_core::serde_json::Map;
use jsonrpc_http_server::jsonrpc_core::{Error, Value};

type MapResult = Result<Map<String, Value>, Error>;

pub fn parse_hold_response(holds: Vec<HoldDb>) -> MapResult {
    Ok(Map::new())
}

pub fn parse_withdrawal_response(withdrawals: Vec<WithdrawNFTDb>) -> MapResult {
    Ok(Map::new())
}

pub fn parse_mint_response(withdrawals: Vec<WithdrawDb>) -> MapResult {
    Ok(Map::new())
}
