use crate::request::define::DefineReq;
use failure::Fail;
use jsonrpc_http_server::jsonrpc_core::serde_json::Map;
use jsonrpc_http_server::jsonrpc_core::{Error, Params, Value};

pub async fn generate_define_cota_smt(params: Params) -> Result<Value, Error> {
    if let Params::Map(map) = params {
        match DefineReq::from_map(map) {
            Ok(_define_req) => {
                let mut response = Map::new();
                response.insert(
                    "smt_root_hash".to_string(),
                    Value::String("root_hash".to_owned()),
                );
                response.insert(
                    "define_smt_entries".to_string(),
                    Value::String("define_smt_entries".to_owned()),
                );
                return Ok(Value::Object(response));
            }
            Err(e) => Ok(Value::String(e.cause().unwrap().to_string())),
        }
    } else {
        Ok(Value::String(
            "Request parameter should be a map".to_owned(),
        ))
    }
}
