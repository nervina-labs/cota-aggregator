use crate::error::Error;
use crate::request::helper::{parse_vec_map, DbParser};
use crate::utils::HexParser;
use jsonrpc_http_server::jsonrpc_core::serde_json::Map;
use jsonrpc_http_server::jsonrpc_core::Value;

#[derive(Clone, Eq, PartialEq)]
pub struct MintWithdrawal {
    pub token_index:    [u8; 4],
    pub state:          u8,
    pub characteristic: [u8; 20],
    pub to_lock_script: Vec<u8>,
}

impl DbParser for MintWithdrawal {
    fn from_map(map: &Map<String, Value>) -> Result<Self, Error> {
        Ok(MintWithdrawal {
            token_index:    map.get_hex_bytes_filed::<4>("token_index")?,
            state:          map.get_hex_bytes_filed::<1>("state")?[0],
            characteristic: map.get_hex_bytes_filed::<20>("characteristic")?,
            to_lock_script: map.get_hex_vec_filed("to_lock_script")?,
        })
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct MintReq {
    pub lock_hash:   [u8; 32],
    pub cota_id:     [u8; 20],
    pub out_point:   [u8; 24],
    pub withdrawals: Vec<MintWithdrawal>,
}

impl MintReq {
    pub fn from_map(map: &Map<String, Value>) -> Result<Self, Error> {
        Ok(MintReq {
            lock_hash:   map.get_hex_bytes_filed::<32>("lock_hash")?,
            cota_id:     map.get_hex_bytes_filed::<20>("cota_id")?,
            out_point:   map.get_hex_bytes_filed::<24>("out_point")?,
            withdrawals: parse_vec_map::<MintWithdrawal>(map, "withdrawals")?,
        })
    }
}
