use super::helper::HexParser;
use super::update::Nft;
use crate::request::helper::{check_secp256k1_batch_master_lock, parse_vec_map, ReqParser};
use crate::utils::error::Error;
use jsonrpc_http_server::jsonrpc_core::serde_json::Map;
use jsonrpc_http_server::jsonrpc_core::Value;

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Claim {
    pub cota_id:     [u8; 20],
    pub token_index: [u8; 4],
}

impl ReqParser for Claim {
    fn from_map(map: &Map<String, Value>) -> Result<Self, Error> {
        Ok(Claim {
            cota_id:     map.get_hex_bytes_field::<20>("cota_id")?,
            token_index: map.get_hex_bytes_field::<4>("token_index")?,
        })
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct ClaimReq {
    pub lock_script:            Vec<u8>,
    pub withdrawal_lock_script: Vec<u8>,
    pub claims:                 Vec<Claim>,
}

impl ClaimReq {
    pub fn from_map(map: &Map<String, Value>) -> Result<Self, Error> {
        let lock_script = map.get_script_field("lock_script")?;
        check_secp256k1_batch_master_lock(&lock_script)?;
        Ok(ClaimReq {
            lock_script,
            withdrawal_lock_script: map.get_hex_vec_field("withdrawal_lock_script")?,
            claims: parse_vec_map::<Claim>(map, "claims")?,
        })
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct IsClaimedReq {
    pub lock_script: Vec<u8>,
    pub cota_id:     [u8; 20],
    pub token_index: [u8; 4],
}

impl IsClaimedReq {
    pub fn from_map(map: &Map<String, Value>) -> Result<Self, Error> {
        Ok(IsClaimedReq {
            lock_script: map.get_script_field("lock_script")?,
            cota_id:     map.get_hex_bytes_field::<20>("cota_id")?,
            token_index: map.get_hex_bytes_field::<4>("token_index")?,
        })
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct ClaimUpdateReq {
    pub lock_script:            Vec<u8>,
    pub withdrawal_lock_script: Vec<u8>,
    pub nfts:                   Vec<Nft>,
}

impl ClaimUpdateReq {
    pub fn from_map(map: &Map<String, Value>) -> Result<Self, Error> {
        let lock_script = map.get_script_field("lock_script")?;
        check_secp256k1_batch_master_lock(&lock_script)?;
        Ok(ClaimUpdateReq {
            lock_script,
            withdrawal_lock_script: map.get_script_field("withdrawal_lock_script")?,
            nfts: parse_vec_map::<Nft>(map, "nfts")?,
        })
    }
}
