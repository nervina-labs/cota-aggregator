pub mod api;
mod config;
mod db;
mod error;
mod request;
mod smt;
mod utils;
pub mod schema;
mod models;

#[macro_use]
extern crate diesel;
extern crate dotenv;

use jsonrpc_http_server::jsonrpc_core::{IoHandler, Params};
use jsonrpc_http_server::ServerBuilder;
use log::info;
use crate::api::*;

const DEFINE_RPC: &'static str = "generate_define_cota_smt";
const MINT_RPC: &'static str = "generate_mint_cota_smt";
const WITHDRAWAL_RPC: &'static str = "generate_withdrawal_cota_smt";
const CLAIM_RPC: &'static str = "generate_claim_cota_smt";
const UPDATE_RPC: &'static str = "generate_update_cota_smt";
const TRANSFER_RPC: &'static str = "generate_transfer_cota_smt";

fn main() {
    env_logger::Builder::from_default_env()
        .format_timestamp(Some(env_logger::fmt::TimestampPrecision::Millis))
        .init();
    let mut io = IoHandler::default();
    io.add_method(DEFINE_RPC, move |params: Params| define_rpc(params));
    io.add_method(MINT_RPC, move |params: Params| mint_rpc(params));
    io.add_method(WITHDRAWAL_RPC, move |params: Params| withdrawal_rpc(params));
    io.add_method(CLAIM_RPC, move |params: Params| claim_rpc(params));
    io.add_method(UPDATE_RPC, move |params: Params| update_rpc(params));
    io.add_method(TRANSFER_RPC, move |params: Params| transfer_rpc(params));

    let server = ServerBuilder::new(io)
        .threads(3)
        .start_http(&"0.0.0.0:3030".parse().unwrap())
        .unwrap();

    info!("Cota aggregator server start");

    server.wait();
}
