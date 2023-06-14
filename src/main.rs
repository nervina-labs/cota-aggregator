#![feature(test)]
#[macro_use]
extern crate diesel;
extern crate dotenv;

use crate::api::*;
use crate::models::helper::init_connection_pool;
use crate::smt::db::db::RocksDB;
use dotenv::dotenv;
use jsonrpc_http_server::jsonrpc_core::serde_json::from_str;
use jsonrpc_http_server::jsonrpc_core::IoHandler;
use jsonrpc_http_server::ServerBuilder;
use lazy_static::lazy_static;
use log::info;
use models::SqlConnectionPool;
use std::env;

pub mod api;
mod business;
mod ckb;
mod entries;
mod models;
mod request;
mod response;
pub mod schema;
mod smt;
mod utils;

#[cfg(all(not(target_env = "msvc"), not(target_os = "macos")))]
#[global_allocator]
static ALLOC: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

lazy_static! {
    static ref ROCKS_DB: RocksDB = RocksDB::default().expect("RocksDB open error");
    static ref POOL: SqlConnectionPool = init_connection_pool();
}

fn main() {
    dotenv().ok();
    env_logger::Builder::from_default_env()
        .format_timestamp(Some(env_logger::fmt::TimestampPrecision::Millis))
        .init();

    if let Ok(dsn) = env::var("SENTRY_DSN") {
        let _guard = sentry::init((dsn, sentry::ClientOptions {
            release: sentry::release_name!(),
            ..Default::default()
        }));
    }

    let mut io = IoHandler::default();
    io.add_method("generate_define_cota_smt", define_rpc);
    io.add_method("generate_mint_cota_smt", mint_rpc);
    io.add_method("generate_claim_cota_smt", claim_rpc);
    io.add_method("generate_update_cota_smt", update_rpc);
    io.add_method("generate_transfer_cota_smt", transfer_rpc);
    io.add_method("generate_withdrawal_cota_smt", withdrawal_rpc);
    io.add_method("generate_claim_update_cota_smt", claim_update_rpc);
    io.add_method("generate_transfer_update_cota_smt", transfer_update_rpc);
    io.add_method("generate_sequential_transfer_cota_smt", sequential_transfer_rpc);
    io.add_method("generate_extension_subkey_smt", extension_subkey_rpc);
    io.add_method("generate_subkey_unlock_smt", subkey_unlock_rpc);
    io.add_method("generate_extension_social_smt", extension_social_rpc);
    io.add_method("generate_social_unlock_smt", social_unlock_rpc);
    io.add_method("get_hold_cota_nft", fetch_hold_rpc);
    io.add_method("get_withdrawal_cota_nft", fetch_withdrawal_rpc);
    io.add_method("get_mint_cota_nft", fetch_mint_rpc);
    io.add_method("is_claimed", is_claimed_rpc);
    io.add_method("get_cota_nft_sender", get_sender_account);
    io.add_method("get_cota_nft_owner", get_owner_account);
    io.add_method("get_define_info", get_define_info);
    io.add_method("get_issuer_info", get_issuer_info);
    io.add_method("get_cota_nft_info", get_cota_nft_info);
    io.add_method("get_joyid_info", get_joyid_info);
    io.add_method("parse_witness", parse_witness);
    io.add_method("get_cota_count", get_cota_count);
    io.add_method("get_history_transactions", get_cota_history_transactions);
    io.add_method("get_transactions_by_block_number", get_txs_by_block_number);
    io.add_method("get_issuer_info_by_cota_id", get_issuer_info_by_cota_id);
    io.add_method("get_aggregator_info", get_aggregator_info);

    let threads: usize = match env::var("THREADS") {
        Ok(thread) => from_str::<usize>(&thread).unwrap(),
        Err(_e) => 3,
    };

    let server = ServerBuilder::new(io)
        .threads(threads)
        .start_http(&"0.0.0.0:3030".parse().unwrap())
        .unwrap();

    let version = env!("CARGO_PKG_VERSION");
    info!("{}", format!("Cota aggregator v{} server start", version));

    server.wait();
}
