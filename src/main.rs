use jsonrpc_http_server::jsonrpc_core::{IoHandler, Params};
use jsonrpc_http_server::ServerBuilder;
use rpc::api::{claim_rpc, define_rpc, mint_rpc, update_rpc, withdrawal_rpc};

const DEFINE_RPC: &'static str = "generate_define_cota_smt";
const MINT_RPC: &'static str = "generate_mint_cota_smt";
const WITHDRAWAL_RPC: &'static str = "generate_withdrawal_cota_smt";
const CLAIM_RPC: &'static str = "generate_claim_cota_smt";
const UPDATE_RPC: &'static str = "generate_update_cota_smt";

fn main() {
    let mut io = IoHandler::default();
    io.add_method(DEFINE_RPC, move |params: Params| define_rpc(params));
    io.add_method(MINT_RPC, move |params: Params| mint_rpc(params));
    io.add_method(WITHDRAWAL_RPC, move |params: Params| withdrawal_rpc(params));
    io.add_method(CLAIM_RPC, move |params: Params| claim_rpc(params));
    io.add_method(UPDATE_RPC, move |params: Params| update_rpc(params));

    let server = ServerBuilder::new(io)
        .threads(3)
        .start_http(&"127.0.0.1:3030".parse().unwrap())
        .unwrap();

    server.wait();
}
