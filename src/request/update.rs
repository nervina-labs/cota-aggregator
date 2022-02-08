use super::helper::HexParser;
use crate::request::helper::{parse_vec_map, ReqParser};
use crate::utils::error::Error;
use jsonrpc_http_server::jsonrpc_core::serde_json::Map;
use jsonrpc_http_server::jsonrpc_core::Value;

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Nft {
    pub cota_id:        [u8; 20],
    pub token_index:    [u8; 4],
    pub state:          u8,
    pub characteristic: [u8; 20],
}

impl ReqParser for Nft {
    fn from_map(map: &Map<String, Value>) -> Result<Self, Error> {
        Ok(Nft {
            cota_id:        map.get_hex_bytes_filed::<20>("cota_id")?,
            token_index:    map.get_hex_bytes_filed::<4>("token_index")?,
            state:          map.get_hex_bytes_filed::<1>("state")?[0],
            characteristic: map.get_hex_bytes_filed::<20>("characteristic")?,
        })
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct UpdateReq {
    pub lock_hash: [u8; 32],
    pub nfts:      Vec<Nft>,
}

impl UpdateReq {
    pub fn from_map(map: &Map<String, Value>) -> Result<Self, Error> {
        Ok(UpdateReq {
            lock_hash: map.get_hex_bytes_filed::<32>("lock_hash")?,
            nfts:      parse_vec_map::<Nft>(map, "nfts")?,
        })
    }
}
