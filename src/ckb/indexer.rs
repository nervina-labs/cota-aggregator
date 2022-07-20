use crate::ckb::constants::{MAINNET_COTA_CODE_HASH, TESTNET_COTA_CODE_HASH};
use crate::utils::error::Error;
use crate::utils::helper::is_ckb_mainnet;
use ckb_jsonrpc_types::{BlockNumber, CellOutput, JsonBytes, OutPoint, Uint32};
use ckb_types::packed::Script;
use ckb_types::prelude::Entity;
use ckb_types::H256;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::{from_str, json, Map, Value};
use std::env;

pub async fn get_cota_smt_root(lock_script: &[u8]) -> Result<Option<[u8; 32]>, Error> {
    let result =
        build_rpc::<CellPagination>("get_cells", Some(generate_params(lock_script)?)).await?;
    if result.objects.is_empty() {
        return Ok(None);
    }
    let cell_data = result.objects.first().unwrap().output_data.as_bytes();
    match cell_data.len() {
        1 => Ok(None),
        33 => {
            let mut ret = [0u8; 32];
            ret.copy_from_slice(&cell_data[1..]);
            Ok(Some(ret))
        }
        _ => Err(Error::CKBIndexerError(
            "CoTA cell data length error".to_owned(),
        )),
    }
}

pub async fn get_indexer_tip_block_number() -> Result<u64, Error> {
    let result = build_rpc::<Tip>("get_tip", None).await?;
    Ok(u64::from(result.block_number))
}

async fn build_rpc<T: DeserializeOwned>(method: &str, params: Option<Value>) -> Result<T, Error> {
    let ckb_indexer_url =
        env::var("CKB_INDEXER").map_err(|_e| Error::Other("CKB_INDEXER must be set".to_owned()))?;

    let mut req_json = Map::new();
    req_json.insert("id".to_owned(), json!("1"));
    req_json.insert("jsonrpc".to_owned(), json!("2.0"));
    req_json.insert("method".to_owned(), json!(method));
    if let Some(param) = params {
        req_json.insert("params".to_owned(), param);
    }

    let client = reqwest::Client::new();

    let resp = client
        .post(ckb_indexer_url)
        .json(&req_json)
        .send()
        .await
        .map_err(|e| Error::Other(format!("CKB Indexer rpc error: {:?}", e.to_string())))?;
    let output = resp
        .json::<jsonrpc_core::response::Output>()
        .await
        .map_err(|e| Error::Other(format!("CKB Indexer rpc error: {:?}", e.to_string())))?;
    match output {
        jsonrpc_core::response::Output::Success(success) => {
            serde_json::from_value::<T>(success.result)
                .map_err(|_e| Error::CKBIndexerError("Parse response error".to_owned()))
        }
        jsonrpc_core::response::Output::Failure(failure) => {
            Err(Error::CKBIndexerError(failure.error.message))
        }
    }
}

fn generate_params(lock_script: &[u8]) -> Result<Value, Error> {
    let lock = Script::from_slice(lock_script)
        .map_err(|_e| Error::Other("Lock script format error".to_owned()))?;
    let hash_type = match lock.hash_type().into() {
        0u8 => "data",
        1u8 => "type",
        2u8 => "data1",
        _ => "0",
    };
    let code_hash = if is_ckb_mainnet() {
        format!("0x{}", MAINNET_COTA_CODE_HASH)
    } else {
        format!("0x{}", TESTNET_COTA_CODE_HASH)
    };

    Ok(json!([
        {
            "script": {
                "code_hash": format!("0x{}", hex::encode(lock.code_hash().as_slice())),
                "hash_type": hash_type,
                "args": format!("0x{}", hex::encode(lock.args().raw_data())),
            },
            "script_type": "lock",
            "filter": {
                "script": {
                    "code_hash": code_hash,
                    "hash_type": "type",
                    "args": "0x",
                },
            }
        },
        "asc",
        "0x1"
    ]))
}

#[derive(Deserialize)]
struct Cell {
    #[serde(skip_deserializing)]
    _output:       CellOutput,
    output_data:   JsonBytes,
    #[serde(skip_deserializing)]
    _out_point:    OutPoint,
    #[serde(skip_deserializing)]
    _block_number: BlockNumber,
    #[serde(skip_deserializing)]
    _tx_index:     Uint32,
}

#[derive(Deserialize)]
struct CellPagination {
    objects:      Vec<Cell>,
    #[serde(skip_deserializing)]
    _last_cursor: JsonBytes,
}

#[derive(Deserialize)]
pub struct Tip {
    #[serde(skip_deserializing)]
    _block_hash:  H256,
    block_number: BlockNumber,
}
