use crate::request::define::DefineReq;
use crate::smt::define::generate_define_smt;
use jsonrpc_http_server::jsonrpc_core::serde_json::Map;
use jsonrpc_http_server::jsonrpc_core::{Error, Params, Value};

pub async fn generate_define_cota_smt(params: Params) -> Result<Value, Error> {
    let mut response = Map::new();
    if let Params::Map(map) = params {
        return match DefineReq::from_map(map) {
            Ok(define_req) => {
                let (smt_root_hash, define_smt_entries) = generate_define_smt(define_req);
                response.insert("smt_root_hash".to_string(), Value::String(smt_root_hash));
                response.insert(
                    "define_smt_entries".to_string(),
                    Value::String(define_smt_entries),
                );
                Ok(Value::Object(response))
            }
            Err(e) => {
                response.insert("error_msg".to_owned(), Value::String(e.to_msg()));
                Ok(Value::Object(response))
            }
        };
    } else {
        response.insert(
            "error_msg".to_owned(),
            Value::String("Request parameter must be an object".to_string()),
        );
        Ok(Value::Object(response))
    }
}
