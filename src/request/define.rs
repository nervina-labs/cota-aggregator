use super::helper::HexParser;
use crate::utils::error::Error;
use jsonrpc_http_server::jsonrpc_core::serde_json::Map;
use jsonrpc_http_server::jsonrpc_core::Value;

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct DefineReq {
    pub lock_hash: [u8; 32],
    pub cota_id:   [u8; 20],
    pub total:     [u8; 4],
    pub issued:    [u8; 4],
    pub configure: u8,
}

impl DefineReq {
    pub fn from_map(map: Map<String, Value>) -> Result<Self, Error> {
        Ok(DefineReq {
            lock_hash: map.get_hex_bytes_filed::<32>("lock_hash")?,
            cota_id:   map.get_hex_bytes_filed::<20>("cota_id")?,
            total:     map.get_hex_bytes_filed::<4>("total")?,
            issued:    map.get_hex_bytes_filed::<4>("issued")?,
            configure: map.get_hex_bytes_filed::<1>("configure")?[0],
        })
    }
}
