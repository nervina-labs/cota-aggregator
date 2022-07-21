use super::helper::HexParser;
use crate::request::helper::{check_secp256k1_batch_master_lock, parse_vec_map, ReqParser};
use crate::request::withdrawal::TransferWithdrawal;
use crate::utils::error::Error;
use jsonrpc_http_server::jsonrpc_core::serde_json::Map;
use jsonrpc_http_server::jsonrpc_core::Value;

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct TransferReq {
    pub lock_script:            Vec<u8>,
    pub withdrawal_lock_script: Vec<u8>,
    pub transfer_out_point:     [u8; 24],
    pub transfers:              Vec<TransferWithdrawal>,
}

impl TransferReq {
    pub fn from_map(map: &Map<String, Value>) -> Result<Self, Error> {
        let lock_script = map.get_script_field("lock_script")?;
        check_secp256k1_batch_master_lock(&lock_script)?;
        Ok(TransferReq {
            lock_script,
            withdrawal_lock_script: map.get_script_field("withdrawal_lock_script")?,
            transfer_out_point: map.get_hex_bytes_field::<24>("transfer_out_point")?,
            transfers: parse_vec_map::<TransferWithdrawal>(map, "transfers")?,
        })
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct TransferUpdate {
    pub cota_id:        [u8; 20],
    pub token_index:    [u8; 4],
    pub to_lock_script: Vec<u8>,
    pub state:          u8,
    pub characteristic: [u8; 20],
}

impl ReqParser for TransferUpdate {
    fn from_map(map: &Map<String, Value>) -> Result<Self, Error> {
        Ok(TransferUpdate {
            cota_id:        map.get_hex_bytes_field::<20>("cota_id")?,
            token_index:    map.get_hex_bytes_field::<4>("token_index")?,
            to_lock_script: map.get_script_field("to_lock_script")?,
            state:          map.get_hex_bytes_field::<1>("state")?[0],
            characteristic: map.get_hex_bytes_field::<20>("characteristic")?,
        })
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct TransferUpdateReq {
    pub lock_script:            Vec<u8>,
    pub withdrawal_lock_script: Vec<u8>,
    pub transfer_out_point:     [u8; 24],
    pub transfers:              Vec<TransferUpdate>,
}

impl TransferUpdateReq {
    pub fn from_map(map: &Map<String, Value>) -> Result<Self, Error> {
        let lock_script = map.get_script_field("lock_script")?;
        check_secp256k1_batch_master_lock(&lock_script)?;
        Ok(TransferUpdateReq {
            lock_script,
            withdrawal_lock_script: map.get_script_field("withdrawal_lock_script")?,
            transfer_out_point: map.get_hex_bytes_field::<24>("transfer_out_point")?,
            transfers: parse_vec_map::<TransferUpdate>(map, "transfers")?,
        })
    }
}
