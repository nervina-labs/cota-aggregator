use super::helper::HexParser;
use crate::request::helper::{parse_vec_map, ReqParser};
use crate::request::withdrawal::TransferWithdrawal;
use crate::utils::error::Error;
use cota_smt::ckb_types::packed::Script;
use cota_smt::ckb_types::prelude::Entity;
use cota_smt::smt::blake2b_256;
use jsonrpc_http_server::jsonrpc_core::serde_json::Map;
use jsonrpc_http_server::jsonrpc_core::Value;

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct TransferReq {
    pub lock_script:            Vec<u8>,
    pub withdrawal_lock_script: Option<Vec<u8>>,
    pub withdrawal_lock_hash:   [u8; 32],
    pub transfer_out_point:     [u8; 24],
    pub transfers:              Vec<TransferWithdrawal>,
}

impl TransferReq {
    pub fn from_map(map: &Map<String, Value>) -> Result<Self, Error> {
        let withdrawal_lock_hash = match map.get("withdrawal_lock_script") {
            Some(_) => {
                let lock = map.get_hex_vec_filed("withdrawal_lock_script")?;
                if Script::from_slice(&lock).is_err() {
                    return Err(Error::RequestParamTypeInvalid("Script".to_string()));
                }
                blake2b_256(&lock)
            }
            None => map
                .get_hex_bytes_filed::<32>("withdrawal_lock_hash")
                .map_err(|_| Error::RequestParamTypeInvalid("Withdrawal lock hash".to_string()))?,
        };

        Ok(TransferReq {
            lock_script: map.get_hex_vec_filed("lock_script")?,
            withdrawal_lock_script: None,
            withdrawal_lock_hash,
            transfer_out_point: map.get_hex_bytes_filed::<24>("transfer_out_point")?,
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
            cota_id:        map.get_hex_bytes_filed::<20>("cota_id")?,
            token_index:    map.get_hex_bytes_filed::<4>("token_index")?,
            to_lock_script: map.get_hex_vec_filed("to_lock_script")?,
            state:          map.get_hex_bytes_filed::<1>("state")?[0],
            characteristic: map.get_hex_bytes_filed::<20>("characteristic")?,
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
        Ok(TransferUpdateReq {
            lock_script:            map.get_hex_vec_filed("lock_script")?,
            withdrawal_lock_script: map.get_hex_vec_filed("withdrawal_lock_script")?,
            transfer_out_point:     map.get_hex_bytes_filed::<24>("transfer_out_point")?,
            transfers:              parse_vec_map::<TransferUpdate>(map, "transfers")?,
        })
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct SequentialTransfer {
    pub withdrawal_lock_hash: [u8; 32],
    pub transfer_out_point:   [u8; 24],
    pub cota_id:              [u8; 20],
    pub token_index:          [u8; 4],
    pub to_lock_script:       Vec<u8>,
}

impl ReqParser for SequentialTransfer {
    fn from_map(map: &Map<String, Value>) -> Result<Self, Error> {
        Ok(SequentialTransfer {
            withdrawal_lock_hash: map.get_hex_bytes_filed::<32>("withdrawal_lock_hash")?,
            transfer_out_point:   map.get_hex_bytes_filed::<24>("transfer_out_point")?,
            cota_id:              map.get_hex_bytes_filed::<20>("cota_id")?,
            token_index:          map.get_hex_bytes_filed::<4>("token_index")?,
            to_lock_script:       map.get_hex_vec_filed("to_lock_script")?,
        })
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct SubKeyUnlock {
    pub alg_index:   u16,
    pub pubkey_hash: [u8; 20],
}

impl SubKeyUnlock {
    pub fn from_map(map: &Map<String, Value>) -> Result<Self, Error> {
        Ok(SubKeyUnlock {
            alg_index:   map.get_u16_filed("alg_index")?,
            pubkey_hash: map.get_hex_bytes_filed::<20>("pubkey_hash")?,
        })
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct SequentialTransferReq {
    pub lock_script: Vec<u8>,
    pub transfers:   Vec<SequentialTransfer>,
    pub subkey:      Option<SubKeyUnlock>,
}

impl SequentialTransferReq {
    pub fn from_map(map: &Map<String, Value>) -> Result<Self, Error> {
        let mut req = SequentialTransferReq {
            lock_script: map.get_hex_vec_filed("lock_script")?,
            transfers:   parse_vec_map::<SequentialTransfer>(map, "transfers")?,
            subkey:      None,
        };
        if let Some(subkey) = map.get("subkey") {
            if subkey.as_object().is_none() {
                return Err(Error::RequestParamTypeInvalid("subkey".to_owned()));
            }
            req.subkey = Some(SubKeyUnlock::from_map(subkey.as_object().unwrap())?)
        }
        Ok(req)
    }
}
