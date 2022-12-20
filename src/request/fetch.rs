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
    pub lock_script: Option<Vec<u8>>,
    pub address:     Option<String>,
}

impl FetchIssuerReq {
    pub fn from_map(map: &Map<String, Value>) -> Result<Self, Error> {
        let lock_script = match map.get("lock_script") {
            Some(_) => {
                let lock = map.get_hex_vec_filed("lock_script")?;
                if Script::from_slice(&lock).is_err() {
                    return Err(Error::RequestParamTypeError("Script".to_string()));
                }
                Some(lock)
            }
            None => None,
        };

        let address = match map.get("address") {
            Some(_) => Some(map.get_str_filed("address")?),
            None => None,
        };
        if lock_script.is_none() && address.is_none() {
            return Err(Error::RequestParamTypeError(
                "lock script and address".to_string(),
            ));
        }
        Ok(FetchIssuerReq {
            lock_script,
            address,
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
            cota_id:     map.get_hex_bytes_filed::<20>("cota_id")?,
            token_index: map.get_hex_bytes_filed::<4>("token_index")?,
            page:        map.get_i64_filed("page")?,
            page_size:   map.get_i64_filed("page_size")?,
        })
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct FetchTxsByBlockNumberReq {
    pub block_number: u64,
}

impl FetchTxsByBlockNumberReq {
    pub fn from_map(map: &Map<String, Value>) -> Result<Self, Error> {
        Ok(FetchTxsByBlockNumberReq {
            block_number: map.get_u64_filed("block_number")?,
        })
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct FetchIssuerInfoReq {
    pub cota_id: [u8; 20],
}

impl FetchIssuerInfoReq {
    pub fn from_map(map: &Map<String, Value>) -> Result<Self, Error> {
        Ok(FetchIssuerInfoReq {
            cota_id: map.get_hex_bytes_filed::<20>("cota_id")?,
        })
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct FetchJoyIDReq {
    pub lock_script: Option<Vec<u8>>,
    pub address:     Option<String>,
}

impl FetchJoyIDReq {
    pub fn from_map(map: &Map<String, Value>) -> Result<Self, Error> {
        let lock_script = match map.get("lock_script") {
            Some(_) => {
                let lock = map.get_hex_vec_filed("lock_script")?;
                if Script::from_slice(&lock).is_err() {
                    return Err(Error::RequestParamTypeError("Script".to_string()));
                }
                Some(lock)
            }
            None => None,
        };

        let address = match map.get("address") {
            Some(_) => Some(map.get_str_filed("address")?),
            None => None,
        };
        if lock_script.is_none() && address.is_none() {
            return Err(Error::RequestParamTypeError(
                "lock script and address".to_string(),
            ));
        }
        Ok(FetchJoyIDReq {
            lock_script,
            address,
        })
    }
}
