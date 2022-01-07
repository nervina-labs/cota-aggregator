use crate::error::Error;
use crate::request::helper::{parse_vec_map, ReqParser};
use crate::utils::HexParser;
use jsonrpc_http_server::jsonrpc_core::serde_json::Map;
use jsonrpc_http_server::jsonrpc_core::Value;

#[derive(Clone, Eq, PartialEq)]
pub struct TransferWithdrawal {
    pub cota_id:        [u8; 20],
    pub token_index:    [u8; 4],
    pub to_lock_script: Vec<u8>,
}

impl ReqParser for TransferWithdrawal {
    fn from_map(map: &Map<String, Value>) -> Result<Self, Error> {
        Ok(TransferWithdrawal {
            cota_id:        map.get_hex_bytes_filed::<20>("cota_id")?,
            token_index:    map.get_hex_bytes_filed::<4>("token_index")?,
            to_lock_script: map.get_hex_vec_filed("to_lock_script")?,
        })
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct WithdrawalReq {
    pub lock_hash:   [u8; 32],
    pub out_point:   [u8; 24],
    pub withdrawals: Vec<TransferWithdrawal>,
}

impl WithdrawalReq {
    pub fn from_map(map: &Map<String, Value>) -> Result<Self, Error> {
        Ok(WithdrawalReq {
            lock_hash:   map.get_hex_bytes_filed::<32>("lock_hash")?,
            out_point:   map.get_hex_bytes_filed::<24>("out_point")?,
            withdrawals: parse_vec_map::<TransferWithdrawal>(map, "withdrawals")?,
        })
    }
}
