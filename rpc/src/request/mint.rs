use crate::error::Error;
use crate::utils::HexParser;
use jsonrpc_http_server::jsonrpc_core::serde_json::Map;
use jsonrpc_http_server::jsonrpc_core::Value;

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct MintWithdrawal {
    pub token_index:    [u8; 4],
    pub configure:      u8,
    pub state:          u8,
    pub characteristic: [u8; 20],
    pub to_lock_hash:   [u8; 32],
}

impl MintWithdrawal {
    pub fn from_map(map: &Map<String, Value>) -> Result<Self, Error> {
        let token_index_vec = map.get_hex_bytes_filed("token_index", 4)?;
        let mut token_index = [0u8; 4];
        token_index.copy_from_slice(&token_index_vec);

        let configure = map.get_hex_bytes_filed("configure", 1)?[0];
        let state = map.get_hex_bytes_filed("state", 1)?[0];

        let characteristic_vec = map.get_hex_bytes_filed("characteristic", 20)?;
        let mut characteristic = [0u8; 20];
        characteristic.copy_from_slice(&characteristic_vec);

        let to_lock_hash_vec = map.get_hex_bytes_filed("to_lock_hash", 32)?;
        let mut to_lock_hash = [0u8; 32];
        to_lock_hash.copy_from_slice(&to_lock_hash_vec);

        Ok(MintWithdrawal {
            token_index,
            configure,
            state,
            characteristic,
            to_lock_hash,
        })
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct MintReq {
    pub lock_hash:   [u8; 32],
    pub cota_id:     [u8; 20],
    pub out_point:   [u8; 72],
    pub withdrawals: Vec<MintWithdrawal>,
}

impl MintReq {
    pub fn from_map(map: &Map<String, Value>) -> Result<Self, Error> {
        let lock_hash_vec = map.get_hex_bytes_filed("lock_hash", 32)?;
        let mut lock_hash = [0u8; 32];
        lock_hash.copy_from_slice(&lock_hash_vec);

        let cota_id_vec = map.get_hex_bytes_filed("cota_id", 20)?;
        let mut cota_id = [0u8; 20];
        cota_id.copy_from_slice(&cota_id_vec);

        let out_point_vec = map.get_hex_bytes_filed("out_point", 72)?;
        let mut out_point = [0u8; 72];
        out_point.copy_from_slice(&out_point_vec);

        let withdrawals_value = map
            .get("withdrawals")
            .ok_or(Error::RequestParamNotFound("withdrawals".to_owned()))?;
        if !withdrawals_value.is_array() {
            return Err(Error::RequestParamTypeError("withdrawals".to_owned()));
        }
        let mut withdrawals: Vec<MintWithdrawal> = Vec::new();
        for withdrawal in withdrawals_value.as_array().unwrap() {
            if !withdrawal.is_object() {
                return Err(Error::RequestParamTypeError("withdrawals".to_owned()));
            }
            withdrawals.push(MintWithdrawal::from_map(withdrawal.as_object().unwrap())?)
        }

        Ok(MintReq {
            lock_hash,
            cota_id,
            out_point,
            withdrawals,
        })
    }
}
