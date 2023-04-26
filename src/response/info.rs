use crate::ckb::indexer::get_indexer_tip_block_number;
use crate::ckb::rpc::get_node_tip_block_number;
use crate::models::withdrawal::nft::WithdrawNFTDb;
use crate::response::helper::Inserter;
use crate::utils::error::Error;
use jsonrpc_http_server::jsonrpc_core::serde_json::{Map, Value};
use serde_json::from_str;
use std::env;

pub async fn generate_aggregator_info(block_number: u64) -> Result<Value, Error> {
    let version = env!("CARGO_PKG_VERSION");
    let is_mainnet: bool = match env::var("IS_MAINNET") {
        Ok(mainnet) => from_str::<bool>(&mainnet).unwrap(),
        Err(_e) => false,
    };
    let mut map = Map::new();
    map.insert_u64("node_block_number", get_node_tip_block_number().await?);
    map.insert_u64(
        "indexer_block_number",
        get_indexer_tip_block_number().await?,
    );
    map.insert_str("version", format!("v{:}", version));
    map.insert_u64("syncer_block_number", block_number);
    map.insert("is_mainnet".to_owned(), Value::Bool(is_mainnet));
    Ok(Value::Object(map))
}

pub fn parse_cota_nft_info_response(
    withdrawals: Option<WithdrawNFTDb>,
    block_number: u64,
) -> Result<Value, Error> {
    let withdrawal = withdrawals.map_or(WithdrawNFTDb::default(), |nft| nft);
    let mut map = Map::new();
    map.insert_hex("state", &[withdrawal.state]);
    map.insert_hex("configure", &[withdrawal.configure]);
    map.insert_hex("characteristic", &withdrawal.characteristic);
    map.insert_u64("block_number", block_number);
    Ok(Value::Object(map))
}
