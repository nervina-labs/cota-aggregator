use crate::request::define::DefineReq;
use crate::request::mint::MintReq;
use crate::smt::define::generate_define_smt;
use crate::smt::mint::generate_mint_smt;
use jsonrpc_http_server::jsonrpc_core::serde_json::Map;
use jsonrpc_http_server::jsonrpc_core::{Error, Params, Value};

pub async fn define_rpc(params: Params) -> Result<Value, Error> {
    let map: Map<String, Value> = Params::parse(params)?;
    let define_req = DefineReq::from_map(map).map_err(|err| err.into())?;
    let response = generate_define_smt(define_req).map_err(|err| err.into())?;
    Ok(Value::Object(response))
}

pub async fn mint_rpc(params: Params) -> Result<Value, Error> {
    let map: Map<String, Value> = Params::parse(params)?;
    let mint_req = MintReq::from_map(&map).map_err(|err| err.into())?;
    let response = generate_mint_smt(mint_req).map_err(|err| err.into())?;
    Ok(Value::Object(response))
}
