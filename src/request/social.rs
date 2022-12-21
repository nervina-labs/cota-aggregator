use super::helper::{parse_vec_map, HexParser, ReqParser};
use crate::utils::error::Error;
use jsonrpc_http_server::jsonrpc_core::serde_json::Map;
use jsonrpc_http_server::jsonrpc_core::Value;

#[derive(Clone, Eq, PartialEq)]
pub struct SocialFriend {
    pub lock_script: Vec<u8>,
    pub pubkey:      Vec<u8>,
    pub signature:   Vec<u8>,
    pub unlock_mode: u8,
    pub alg_index:   u16,
}

impl ReqParser for SocialFriend {
    fn from_map(map: &Map<String, Value>) -> Result<Self, Error> {
        Ok(SocialFriend {
            lock_script: map.get_hex_vec_filed("lock_script")?,
            pubkey:      map.get_hex_vec_filed("pubkey")?,
            signature:   map.get_hex_vec_filed("signature")?,
            unlock_mode: map.get_u8_filed("unlock_mode")?,
            alg_index:   map.get_u16_filed("alg_index")?,
        })
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct SocialUnlockReq {
    pub lock_script: Vec<u8>,
    pub friends:     Vec<SocialFriend>,
}

impl SocialUnlockReq {
    pub fn from_map(map: &Map<String, Value>) -> Result<Self, Error> {
        Ok(SocialUnlockReq {
            lock_script: map.get_hex_vec_filed("lock_script")?,
            friends:     parse_vec_map::<SocialFriend>(map, "friends")?,
        })
    }
}
