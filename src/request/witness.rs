use super::helper::HexParser;
use crate::utils::error::Error;
use jsonrpc_http_server::jsonrpc_core::serde_json::Map;
use jsonrpc_http_server::jsonrpc_core::Value;

#[derive(Clone, Eq, PartialEq)]
pub struct WitnessReq {
    pub witness: Vec<u8>,
    pub version: u8,
}

impl WitnessReq {
    pub fn from_map(map: &Map<String, Value>) -> Result<Self, Error> {
        Ok(WitnessReq {
            witness: map.get_hex_vec_field("witness")?,
            version: map.get_u8_field("version")?,
        })
    }
}
