use crate::business::helper::address_from_script;
use crate::models::class::ClassInfoDb;
use crate::models::withdrawal::nft::WithdrawNFTDb;
use crate::response::helper::Inserter;
use crate::utils::error::Error;
use ckb_types::prelude::Entity;
use cota_smt::smt::H256;
use cota_smt::transfer::WithdrawalCotaNFTV1Entries;
use jsonrpc_http_server::jsonrpc_core::serde_json::Map;
use jsonrpc_http_server::jsonrpc_core::Value;

use super::helper::parse_json_err;

pub fn parse_withdrawal_response(
    withdrawals: Vec<(WithdrawNFTDb, Option<ClassInfoDb>)>,
    total: i64,
    page_size: i64,
    block_number: u64,
) -> Result<Value, Error> {
    let mut nfts: Vec<Value> = Vec::new();
    for withdrawal in withdrawals {
        nfts.push(parse_withdrawal_value(withdrawal)?);
    }
    let mut map = Map::new();
    map.insert_i64("total", total);
    map.insert_i64("page_size", page_size);
    map.insert_u64("block_number", block_number);
    map.insert_array("nfts", nfts);
    Ok(Value::Object(map))
}

fn parse_withdrawal_value(
    (withdrawal, class_info): (WithdrawNFTDb, Option<ClassInfoDb>),
) -> Result<Value, Error> {
    let mut map = Map::new();
    map.insert_hex("cota_id", &withdrawal.cota_id);
    map.insert_hex("token_index", &withdrawal.token_index);
    map.insert_hex("state", &[withdrawal.state]);
    map.insert_hex("configure", &[withdrawal.configure]);
    map.insert_hex("characteristic", &withdrawal.characteristic);

    let class = class_info.map_or(ClassInfoDb::default(), |class| class);
    let class_json = serde_json::to_string(&class).map_err(parse_json_err)?;
    let mut class_map: Map<String, Value> =
        serde_json::from_str(&class_json).map_err(parse_json_err)?;

    map.append(&mut class_map);
    Ok(Value::Object(map))
}

pub fn parse_withdrawal_smt(
    (root_hash, withdrawal_entries): (H256, WithdrawalCotaNFTV1Entries),
    block_number: u64,
) -> Value {
    let withdrawal_entry = hex::encode(withdrawal_entries.as_slice());
    let withdrawal_root_hash = hex::encode(root_hash.as_slice());
    let mut map = Map::new();
    map.insert_str("smt_root_hash", withdrawal_root_hash);
    map.insert_str("withdrawal_smt_entry", withdrawal_entry);
    map.insert_u64("block_number", block_number);
    Value::Object(map)
}

pub fn parse_sender_response(
    sender_account: Option<(String, Vec<u8>)>,
    block_number: u64,
) -> Result<Value, Error> {
    let mut map = Map::new();
    match sender_account {
        Some((lock_hash, lock_script)) => {
            map.insert_str("sender_lock_hash", format!("0x{}", lock_hash));
            map.insert_str("sender_address", address_from_script(&lock_script)?);
        }
        None => {
            map.insert_null("sender_lock_hash");
            map.insert_null("sender_address");
        }
    };
    map.insert_u64("block_number", block_number);
    Ok(Value::Object(map))
}
