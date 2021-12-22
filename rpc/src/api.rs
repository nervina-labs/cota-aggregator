use crate::smt::generate_registry_smt;
use crate::utils::{check_request_params, parse_values};
use jsonrpc_http_server::jsonrpc_core::serde_json::Map;
use jsonrpc_http_server::jsonrpc_core::{Error, Params, Value};

pub async fn generate_registry_cota_smt(params: Params) -> Result<Value, Error> {
    if let Params::Array(array) = params {
        if let Some(error) = check_request_params(array.clone()) {
            return Ok(error);
        }

        let parameters = parse_values(array);

        let (root_hash, registry_entries_hex) = generate_registry_smt(parameters);

        let mut response = Map::new();
        response.insert("smt_root_hash".to_string(), Value::String(root_hash));
        response.insert(
            "registry_smt_entries".to_string(),
            Value::String(registry_entries_hex),
        );

        return Ok(Value::Object(response));
    }
    Ok(Value::String(
        "Request parameter should be an array".to_owned(),
    ))
}
