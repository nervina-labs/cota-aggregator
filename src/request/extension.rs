use super::helper::HexParser;
use crate::utils::error::Error;
use jsonrpc_http_server::jsonrpc_core::serde_json::Map;
use jsonrpc_http_server::jsonrpc_core::Value;

#[derive(Clone, Eq, PartialEq)]
pub struct ExtensionReq {
    pub lock_script: Vec<u8>,
}

impl ExtensionReq {
    pub fn from_map(map: &Map<String, Value>) -> Result<Self, Error> {
        Ok(ExtensionReq {
            lock_script: map.get_hex_vec_filed("lock_script")?,
        })
    }
}
