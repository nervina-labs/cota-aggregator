use super::helper::{parse_vec_bytes, parse_vec_map, HexParser, ReqParser};
use crate::utils::error::Error;
use jsonrpc_http_server::jsonrpc_core::serde_json::Map;
use jsonrpc_http_server::jsonrpc_core::Value;

#[derive(Clone, Eq, PartialEq)]
pub struct ExtSubkey {
    pub ext_data:    u32,
    pub alg_index:   u16,
    pub pubkey_hash: [u8; 20],
}

impl ReqParser for ExtSubkey {
    fn from_map(map: &Map<String, Value>) -> Result<Self, Error> {
        Ok(ExtSubkey {
            ext_data:    map.get_u32_filed("ext_data")?,
            alg_index:   map.get_u16_filed("alg_index")?,
            pubkey_hash: map.get_hex_bytes_filed::<20>("pubkey_hash")?,
        })
    }
}

// ext_action: 0xF0 => Add; 0xF1 => Update
pub const EXT_ACTION_ADD: u8 = 0xF0;
pub const EXT_ACTION_UPDATE: u8 = 0xF1;

#[derive(Clone, Eq, PartialEq)]
pub struct ExtSubkeysReq {
    pub lock_script: Vec<u8>,
    pub ext_action:  u8,
    pub subkeys:     Vec<ExtSubkey>,
}

impl ExtSubkeysReq {
    pub fn from_map(map: &Map<String, Value>) -> Result<Self, Error> {
        let ext_action = map.get_u8_filed("ext_action").unwrap_or(0xF0);
        if ext_action != EXT_ACTION_ADD && ext_action != EXT_ACTION_UPDATE {
            return Err(Error::CKBRPCInvalid("Extension action error".to_string()));
        }
        Ok(ExtSubkeysReq {
            lock_script: map.get_hex_vec_filed("lock_script")?,
            subkeys: parse_vec_map::<ExtSubkey>(map, "subkeys")?,
            ext_action,
        })
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct ExtSocialReq {
    pub lock_script:   Vec<u8>,
    pub ext_action:    u8,
    pub recovery_mode: u8,
    pub must:          u8,
    pub total:         u8,
    pub signers:       Vec<Vec<u8>>,
}

impl ExtSocialReq {
    pub fn from_map(map: &Map<String, Value>) -> Result<Self, Error> {
        let ext_action = map.get_u8_filed("ext_action")?;
        if ext_action != EXT_ACTION_ADD && ext_action != EXT_ACTION_UPDATE {
            return Err(Error::CKBRPCInvalid("Extension action error".to_string()));
        }
        let must = map.get_u8_filed("must")?;
        let total = map.get_u8_filed("total")?;
        let signers = parse_vec_bytes(map, "signers")?;
        let signers_len = signers.len() as u8;
        if signers_len != total || must > total {
            return Err(Error::CKBRPCInvalid(
                "Signers length, must and total error".to_string(),
            ));
        }
        Ok(ExtSocialReq {
            lock_script: map.get_hex_vec_filed("lock_script")?,
            recovery_mode: map.get_u8_filed("recovery_mode")?,
            ext_action,
            must,
            total,
            signers,
        })
    }
}
