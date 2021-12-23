use crate::request::define::DefineReq;
use crate::smt::define::generate_define_smt;
use failure::Fail;
use jsonrpc_http_server::jsonrpc_core::serde_json::Map;
use jsonrpc_http_server::jsonrpc_core::{Error, Params, Value};

pub async fn generate_define_cota_smt(params: Params) -> Result<Value, Error> {
    if let Params::Map(map) = params {
        match DefineReq::from_map(map) {
            Ok(define_req) => {
                let (smt_root_hash, define_smt_entries) = generate_define_smt(define_req);
                let mut response = Map::new();
                response.insert("smt_root_hash".to_string(), Value::String(smt_root_hash));
                response.insert(
                    "define_smt_entries".to_string(),
                    Value::String(define_smt_entries),
                );
                return Ok(Value::Object(response));
            }
            Err(e) => Ok(Value::String(e.cause().unwrap().to_string())),
        }
    } else {
        Ok(Value::String(
            "Request parameter should be an object".to_owned(),
        ))
    }
}
