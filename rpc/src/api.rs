use crate::error::Error as SmtError;
use crate::request::define::DefineReq;
use crate::request::mint::MintReq;
use crate::smt::define::generate_define_smt;
use crate::smt::mint::generate_mint_smt;
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

fn parse_smt_error(e: SmtError) -> Value {
    let mut response = Map::new();
    response.insert("error_msg".to_owned(), Value::String(e.to_msg()));
    Value::Object(response)
}

pub async fn generate_mint_cota_smt(params: Params) -> Result<Value, Error> {
    let mut response = Map::new();
    if let Params::Map(map) = params {
        return match MintReq::from_map(&map) {
            Ok(mint_req) => match generate_mint_smt(mint_req) {
                Ok((smt_root_hash, define_smt_entries)) => {
                    response.insert("smt_root_hash".to_string(), Value::String(smt_root_hash));
                    response.insert(
                        "mint_smt_entries".to_string(),
                        Value::String(define_smt_entries),
                    );
                    Ok(Value::Object(response))
                }
                Err(e) => Ok(parse_smt_error(e)),
            },
            Err(e) => Ok(parse_smt_error(e)),
        };
    } else {
        response.insert(
            "error_msg".to_owned(),
            Value::String("Request parameter must be an object".to_string()),
        );
        Ok(Value::Object(response))
    }
}
