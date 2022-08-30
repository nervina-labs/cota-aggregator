use super::helper::Inserter;
use crate::models::class::ClassInfoDb;
use crate::models::withdrawal::nft::WithdrawDb;
use ckb_types::prelude::Entity;
use cota_smt::mint::MintCotaNFTV1Entries;
use cota_smt::smt::H256;
use jsonrpc_http_server::jsonrpc_core::serde_json::Map;
use jsonrpc_http_server::jsonrpc_core::Value;

pub fn parse_mint_response(
    withdrawals: Vec<(WithdrawDb, Option<ClassInfoDb>)>,
    total: i64,
    page_size: i64,
    block_number: u64,
) -> Map<String, Value> {
    let nfts: Vec<Value> = withdrawals.into_iter().map(parse_mint_value).collect();
    let mut map = Map::new();
    map.insert_i64("total", total);
    map.insert_i64("page_size", page_size);
    map.insert_u64("block_number", block_number);
    map.insert_array("nfts", nfts);
    map
}

fn parse_mint_value((withdrawal, class_info): (WithdrawDb, Option<ClassInfoDb>)) -> Value {
    let mut map = Map::new();
    map.insert_hex("cota_id", &withdrawal.cota_id);
    map.insert_hex("token_index", &withdrawal.token_index);
    map.insert_hex("state", &[withdrawal.state]);
    map.insert_hex("configure", &[withdrawal.configure]);
    map.insert_hex("characteristic", &withdrawal.characteristic);
    map.insert_hex("receiver_lock", &withdrawal.receiver_lock_script);
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

pub fn parse_mint_smt(
    (root_hash, mint_entries): (H256, MintCotaNFTV1Entries),
    block_number: u64,
) -> Map<String, Value> {
    let mint_entry = hex::encode(mint_entries.as_slice());
    let mint_root_hash = hex::encode(root_hash.as_slice());
    let mut map = Map::new();
    map.insert_str("smt_root_hash", mint_root_hash);
    map.insert_str("mint_smt_entry", mint_entry);
    map.insert_u64("block_number", block_number);
    map
}
