use crate::error::Error;
use crate::request::helper::{parse_vec_map, DbParser};
use crate::utils::HexParser;
use jsonrpc_http_server::jsonrpc_core::serde_json::Map;
use jsonrpc_http_server::jsonrpc_core::Value;

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Claim {
    pub cota_id:     [u8; 20],
    pub token_index: [u8; 4],
}

impl DbParser for Claim {
    fn from_map(map: &Map<String, Value>) -> Result<Self, Error> {
        Ok(Claim {
            cota_id:     map.get_hex_bytes_filed::<20>("cota_id")?,
            token_index: map.get_hex_bytes_filed::<4>("token_index")?,
        })
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct ClaimReq {
    pub lock_script:          Vec<u8>,
    pub withdrawal_lock_hash: [u8; 32],
    pub claims:               Vec<Claim>,
}

impl ClaimReq {
    pub fn from_map(map: &Map<String, Value>) -> Result<Self, Error> {
        Ok(ClaimReq {
            lock_script:          map.get_hex_vec_filed("lock_script")?,
            withdrawal_lock_hash: map.get_hex_bytes_filed::<32>("withdrawal_lock_hash")?,
            claims:               parse_vec_map::<Claim>(map, "claims")?,
        })
    }
}
