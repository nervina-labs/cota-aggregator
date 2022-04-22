use super::helper::HexParser;
use crate::utils::error::Error;
use cota_smt::ckb_types::packed::Script;
use cota_smt::ckb_types::prelude::Entity;
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
        let lock_script = map.get_hex_vec_filed("lock_script")?;
        if Script::from_slice(&lock_script).is_err() {
            return Err(Error::RequestParamTypeError("Script".to_string()));
        }
        let cota_id = match map.get("cota_id") {
            Some(_) => Some(map.get_hex_bytes_filed::<20>("cota_id")?),
            None => None,
        };
        Ok(FetchReq {
            lock_script,
            page: map.get_i64_filed("page")?,
            page_size: map.get_i64_filed("page_size")?,
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
        let lock_script = map.get_hex_vec_filed("lock_script")?;
        if Script::from_slice(&lock_script).is_err() {
            return Err(Error::RequestParamTypeError("Script".to_string()));
        }
        Ok(FetchIssuerReq { lock_script })
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct FetchCountReq {
    pub lock_script: Vec<u8>,
    pub cota_id:     [u8; 20],
}

impl FetchCountReq {
    pub fn from_map(map: &Map<String, Value>) -> Result<Self, Error> {
        let lock_script = map.get_hex_vec_filed("lock_script")?;
        if Script::from_slice(&lock_script).is_err() {
            return Err(Error::RequestParamTypeError("Script".to_string()));
        }

        Ok(FetchCountReq {
            lock_script,
            cota_id: map.get_hex_bytes_filed::<20>("cota_id")?,
        })
    }
}
