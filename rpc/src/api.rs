use crate::request::define::DefineReq;
use crate::request::mint::MintReq;
use crate::smt::define::generate_define_smt;
use crate::smt::mint::generate_mint_smt;
use jsonrpc_http_server::jsonrpc_core::serde_json::Map;
use jsonrpc_http_server::jsonrpc_core::{Error, Params, Value};

fn parse_smt_error(err_msg: String) -> Value {
    let mut response = Map::new();
    response.insert("err_msg".to_owned(), Value::String(err_msg));
    Value::Object(response)
}

pub async fn generate_define_cota_smt(params: Params) -> Result<Value, Error> {
    if let Params::Map(map) = params {
        return match DefineReq::from_map(map) {
            Ok(define_req) => {
                let (smt_root_hash, define_smt_entries) = generate_define_smt(define_req);
                let mut response = Map::new();
                response.insert("smt_root_hash".to_string(), smt_root_hash);
                response.insert("define_smt_entries".to_string(), define_smt_entries);
                Ok(Value::Object(response))
            }
            Err(e) => Ok(parse_smt_error(e.to_msg())),
        };
    } else {
        Ok(parse_smt_error(
            "Request parameter must be an object".to_owned(),
        ))
    }
}

pub async fn generate_mint_cota_smt(params: Params) -> Result<Value, Error> {
    if let Params::Map(map) = params {
        return match MintReq::from_map(&map) {
            Ok(mint_req) => match generate_mint_smt(mint_req) {
                Ok((smt_root_hash, define_smt_entries)) => {
                    let mut response = Map::new();
                    response.insert("smt_root_hash".to_string(), smt_root_hash);
                    response.insert("mint_smt_entries".to_string(), define_smt_entries);
                    Ok(Value::Object(response))
                }
                Err(e) => Ok(parse_smt_error(e.to_msg())),
            },
            Err(e) => Ok(parse_smt_error(e.to_msg())),
        };
    } else {
        Ok(parse_smt_error(
            "Request parameter must be an object".to_owned(),
        ))
    }
}
