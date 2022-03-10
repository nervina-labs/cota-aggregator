#[macro_use]
extern crate diesel;
extern crate dotenv;

use crate::api::*;
use jsonrpc_http_server::jsonrpc_core::IoHandler;
use jsonrpc_http_server::ServerBuilder;
use log::info;

pub mod api;
mod models;
mod request;
mod response;
pub mod schema;
mod smt;
mod utils;

fn main() {
    env_logger::Builder::from_default_env()
        .format_timestamp(Some(env_logger::fmt::TimestampPrecision::Millis))
        .init();
    let mut io = IoHandler::default();
    io.add_method("generate_define_cota_smt", define_rpc);
    io.add_method("generate_mint_cota_smt", mint_rpc);
    io.add_method("generate_withdrawal_cota_smt", withdrawal_rpc);
    io.add_method("generate_claim_cota_smt", claim_rpc);
    io.add_method("generate_update_cota_smt", update_rpc);
    io.add_method("generate_transfer_cota_smt", transfer_rpc);
    io.add_method("generate_claim_update_cota_smt", claim_update_rpc);
    io.add_method("generate_transfer_update_cota_smt", transfer_update_rpc);
    io.add_method("get_hold_cota_nft", fetch_hold_rpc);
    io.add_method("get_withdrawal_cota_nft", fetch_withdrawal_rpc);
    io.add_method("get_mint_cota_nft", fetch_mint_rpc);
    io.add_method("is_claimed", is_claimed_rpc);
    io.add_method("get_cota_nft_sender", get_sender_lock_hash);

    let server = ServerBuilder::new(io)
        .threads(50)
        .start_http(&"0.0.0.0:3030".parse().unwrap())
        .unwrap();

    info!("Cota aggregator server start");

    server.wait();
}
