use crate::error::Error;
use crate::utils::HexParser;
use jsonrpc_http_server::jsonrpc_core::serde_json::Map;
use jsonrpc_http_server::jsonrpc_core::Value;

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct DefineReq {
    lock_hash: [u8; 32],
    cota_id:   [u8; 20],
    total:     [u8; 4],
    issued:    [u8; 4],
    configure: u8,
}

impl DefineReq {
    pub fn from_map(map: Map<String, Value>) -> Result<Self, Error> {
        let lock_hash_vec = map.get_hex_bytes_filed("lock_hash", 32)?;
        let mut lock_hash = [0u8; 32];
        lock_hash.copy_from_slice(&lock_hash_vec);

        let cota_id_vec = map.get_hex_bytes_filed("cota_id", 20)?;
        let mut cota_id = [0u8; 20];
        cota_id.copy_from_slice(&cota_id_vec);

        let total_vec = map.get_hex_bytes_filed("total", 4)?;
        let mut total = [0u8; 4];
        total.copy_from_slice(&total_vec);

        let issued_vec = map.get_hex_bytes_filed("issued", 4)?;
        let mut issued = [0u8; 4];
        issued.copy_from_slice(&issued_vec);

        let configure = map.get_hex_bytes_filed("configure", 1)?[0];

        Ok(DefineReq {
            lock_hash,
            cota_id,
            total,
            issued,
            configure,
        })
    }
}
