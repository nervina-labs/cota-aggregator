use super::helper::HexParser;
use crate::utils::error::Error;
use jsonrpc_http_server::jsonrpc_core::serde_json::Map;
use jsonrpc_http_server::jsonrpc_core::Value;

#[derive(Clone, Eq, PartialEq)]
pub struct FetchReq {
    pub lock_script: Vec<u8>,
    pub page:        i64,
    pub page_size:   i64,
    pub cota_id:     Option<[u8; 20]>,
}

impl FetchReq {
    pub fn from_map(map: &Map<String, Value>) -> Result<Self, Error> {
        let lock_script = map.get_script_field("lock_script")?;
        let cota_id = match map.get("cota_id") {
            Some(_) => Some(map.get_hex_bytes_field::<20>("cota_id")?),
            None => None,
        };
        Ok(FetchReq {
            lock_script,
            page: map.get_i64_field("page")?,
            page_size: map.get_i64_field("page_size")?,
            cota_id,
        })
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct FetchIssuerReq {
    pub lock_script: Vec<u8>,
}

impl FetchIssuerReq {
    pub fn from_map(map: &Map<String, Value>) -> Result<Self, Error> {
        Ok(FetchIssuerReq {
            lock_script: map.get_script_field("lock_script")?,
        })
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct FetchCountReq {
    pub lock_script: Vec<u8>,
    pub cota_id:     [u8; 20],
}

impl FetchCountReq {
    pub fn from_map(map: &Map<String, Value>) -> Result<Self, Error> {
        Ok(FetchCountReq {
            lock_script: map.get_script_field("lock_script")?,
            cota_id:     map.get_hex_bytes_field::<20>("cota_id")?,
        })
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct FetchHistoryTxsReq {
    pub cota_id:     [u8; 20],
    pub token_index: [u8; 4],
    pub page:        i64,
    pub page_size:   i64,
}

impl FetchHistoryTxsReq {
    pub fn from_map(map: &Map<String, Value>) -> Result<Self, Error> {
        Ok(FetchHistoryTxsReq {
            cota_id:     map.get_hex_bytes_field::<20>("cota_id")?,
            token_index: map.get_hex_bytes_field::<4>("token_index")?,
            page:        map.get_i64_field("page")?,
            page_size:   map.get_i64_field("page_size")?,
        })
    }
}
