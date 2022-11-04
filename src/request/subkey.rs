use super::helper::HexParser;
use crate::utils::error::Error;
use jsonrpc_http_server::jsonrpc_core::serde_json::Map;
use jsonrpc_http_server::jsonrpc_core::Value;

#[derive(Clone, Eq, PartialEq)]
pub struct SubKeyUnlockReq {
    pub lock_script: Vec<u8>,
    pub pubkey_hash: [u8; 20],
}

impl SubKeyUnlockReq {
    pub fn from_map(map: &Map<String, Value>) -> Result<Self, Error> {
        Ok(SubKeyUnlockReq {
            lock_script: map.get_hex_vec_filed("lock_script")?,
            pubkey_hash: map.get_hex_bytes_filed::<20>("pubkey_hash")?,
        })
    }
}
