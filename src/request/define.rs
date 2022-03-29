use super::helper::HexParser;
use crate::utils::error::Error;
use jsonrpc_http_server::jsonrpc_core::serde_json::Map;
use jsonrpc_http_server::jsonrpc_core::Value;

#[derive(Clone, Eq, PartialEq)]
pub struct DefineReq {
    pub lock_script: Vec<u8>,
    pub cota_id:     [u8; 20],
    pub total:       [u8; 4],
    pub issued:      [u8; 4],
    pub configure:   u8,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct DefineInfoReq {
    pub cota_id: [u8; 20],
}

impl DefineReq {
    pub fn from_map(map: &Map<String, Value>) -> Result<Self, Error> {
        Ok(DefineReq {
            lock_script: map.get_hex_vec_filed("lock_script")?,
            cota_id:     map.get_hex_bytes_filed::<20>("cota_id")?,
            total:       map.get_hex_bytes_filed::<4>("total")?,
            issued:      map.get_hex_bytes_filed::<4>("issued")?,
            configure:   map.get_hex_bytes_filed::<1>("configure")?[0],
        })
    }
}

impl DefineInfoReq {
    pub fn from_map(map: &Map<String, Value>) -> Result<Self, Error> {
        Ok(DefineInfoReq {
            cota_id: map.get_hex_bytes_filed::<20>("cota_id")?,
        })
    }
}
