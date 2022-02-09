use super::helper::HexParser;
use crate::utils::error::Error;
use cota_smt::ckb_types::packed::Script;
use cota_smt::ckb_types::prelude::Entity;
use jsonrpc_http_server::jsonrpc_core::serde_json::Map;
use jsonrpc_http_server::jsonrpc_core::Value;

#[derive(Clone, Eq, PartialEq)]
pub struct FetchReq {
    pub lock_script: Vec<u8>,
}

impl FetchReq {
    pub fn from_map(map: &Map<String, Value>) -> Result<Self, Error> {
        let lock_script = map.get_hex_vec_filed("lock_script")?;
        if Script::from_slice(&lock_script).is_err() {
            return Err(Error::RequestParamTypeError("Script".to_string()));
        }
        Ok(FetchReq { lock_script })
    }
}
