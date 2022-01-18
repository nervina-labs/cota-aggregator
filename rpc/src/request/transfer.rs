use crate::error::Error;
use crate::request::helper::parse_vec_map;
use crate::request::withdrawal::TransferWithdrawal;
use crate::utils::HexParser;
use jsonrpc_http_server::jsonrpc_core::serde_json::Map;
use jsonrpc_http_server::jsonrpc_core::Value;

#[derive(Clone, Eq, PartialEq)]
pub struct TransferReq {
    pub lock_script:          Vec<u8>,
    pub withdrawal_lock_hash: [u8; 32],
    pub transfer_out_point:   [u8; 24],
    pub transfers:            Vec<TransferWithdrawal>,
}

impl TransferReq {
    pub fn from_map(map: &Map<String, Value>) -> Result<Self, Error> {
        Ok(TransferReq {
            lock_script:          map.get_hex_vec_filed("lock_script")?,
            withdrawal_lock_hash: map.get_hex_bytes_filed::<32>("withdrawal_lock_hash")?,
            transfer_out_point:   map.get_hex_bytes_filed::<24>("transfer_out_point")?,
            transfers:            parse_vec_map::<TransferWithdrawal>(map, "transfers")?,
        })
    }
}
