use crate::models::class::ClassInfoDb;
use crate::models::withdrawal::WithdrawNFTDb;
use crate::response::helper::Inserter;
use ckb_types::prelude::Entity;
use cota_smt::smt::H256;
use cota_smt::transfer::WithdrawalCotaNFTV1Entries;
use jsonrpc_http_server::jsonrpc_core::serde_json::Map;
use jsonrpc_http_server::jsonrpc_core::Value;

pub fn parse_withdrawal_response(
    withdrawals: Vec<(WithdrawNFTDb, Option<ClassInfoDb>)>,
    total: i64,
    page_size: i64,
    block_number: u64,
) -> Map<String, Value> {
    let nfts: Vec<Value> = withdrawals
        .into_iter()
        .map(parse_withdrawal_value)
        .collect();
    let mut map = Map::new();
    map.insert_i64("total", total);
    map.insert_i64("page_size", page_size);
    map.insert_u64("block_number", block_number);
    map.insert_array("nfts", nfts);
    map
}

fn parse_withdrawal_value((withdrawal, class_info): (WithdrawNFTDb, Option<ClassInfoDb>)) -> Value {
    let mut map = Map::new();
    map.insert_hex("cota_id", &withdrawal.cota_id);
    map.insert_hex("token_index", &withdrawal.token_index);
    map.insert_hex("state", &[withdrawal.state]);
    map.insert_hex("configure", &[withdrawal.configure]);
    map.insert_hex("characteristic", &withdrawal.characteristic);
    match class_info {
        Some(class) => {
            map.insert_str("name", class.name);
            map.insert_str("description", class.description);
            map.insert_str("image", class.image);
            map.insert_str("audio", class.audio);
            map.insert_str("video", class.video);
            map.insert_str("model", class.model);
            map.insert_str("meta_characteristic", class.characteristic);
            map.insert_str("properties", class.properties);
        }
        None => {
            map.insert_null("name");
            map.insert_null("description");
            map.insert_null("image");
            map.insert_null("audio");
            map.insert_null("video");
            map.insert_null("model");
            map.insert_null("meta_characteristic");
            map.insert_null("properties");
        }
    }
    Value::Object(map)
}

pub fn parse_withdrawal_smt(
    (root_hash, withdrawal_entries): (H256, WithdrawalCotaNFTV1Entries),
    block_number: u64,
) -> Map<String, Value> {
    let withdrawal_entry = hex::encode(withdrawal_entries.as_slice());
    let withdrawal_root_hash = hex::encode(root_hash.as_slice());
    let mut map = Map::new();
    map.insert_str("smt_root_hash", withdrawal_root_hash);
    map.insert_str("withdrawal_smt_entry", withdrawal_entry);
    map.insert_u64("block_number", block_number);
    map
}

pub fn parse_sender_response(
    sender_lock_hash: Option<String>,
    block_number: u64,
) -> Map<String, Value> {
    let mut map = Map::new();
    match sender_lock_hash {
        Some(lock_hash) => map.insert_str("sender_lock_hash", format!("0x{}", lock_hash)),
        None => map.insert_null("sender_lock_hash"),
    };
    map.insert_u64("block_number", block_number);
    map
}
