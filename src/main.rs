use crate::utils::check_request_params;
use jsonrpc_http_server::jsonrpc_core::serde_json::Map;
use jsonrpc_http_server::jsonrpc_core::{IoHandler, Params, Value};
use jsonrpc_http_server::ServerBuilder;

mod utils;

fn main() {
    let mut io = IoHandler::default();
    io.add_method("generate_mint_compact_nft", |params: Params| async move {
        match params {
            Params::Array(values) => {
                println!("params: {:?}", values);
                if let Some(error) = check_request_params(values.clone()) {
                    return Ok(error);
                }

                let mut response = Map::new();
                response.insert(
                    "registry_entries".to_string(),
                    Value::String("generate_mint_compact_nft".to_string()),
                );
                response.insert(
                    "smt_root_hash".to_string(),
                    Value::String("smt_root_hash".to_string()),
                );

                Ok(Value::Object(response))
            }
            _ => Ok(Value::String("Request parameter format error".to_owned())),
        }
    });

    let server = ServerBuilder::new(io)
        .threads(3)
        .start_http(&"127.0.0.1:3030".parse().unwrap())
        .unwrap();

    server.wait();
}
