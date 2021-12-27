use jsonrpc_http_server::jsonrpc_core::{IoHandler, Params};
use jsonrpc_http_server::ServerBuilder;
use rpc::api::{define_rpc, mint_rpc};

const DEFINE_RPC: &'static str = "generate_define_cota_smt";
const MINT_RPC: &'static str = "generate_mint_cota_smt";

fn main() {
    let mut io = IoHandler::default();
    io.add_method(DEFINE_RPC, move |params: Params| define_rpc(params));
    io.add_method(MINT_RPC, move |params: Params| mint_rpc(params));

    let server = ServerBuilder::new(io)
        .threads(3)
        .start_http(&"127.0.0.1:3030".parse().unwrap())
        .unwrap();

    server.wait();
}
