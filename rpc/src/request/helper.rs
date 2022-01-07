use crate::error::Error;
use jsonrpc_http_server::jsonrpc_core::serde_json::Map;
use jsonrpc_http_server::jsonrpc_core::Value;

pub trait ReqParser: Sized {
    fn from_map(map: &Map<String, Value>) -> Result<Self, Error>;
}

pub fn parse_vec_map<T: ReqParser>(map: &Map<String, Value>, key: &str) -> Result<Vec<T>, Error> {
    let value = map
        .get(key)
        .ok_or(Error::RequestParamNotFound(key.to_owned()))?;
    if !value.is_array() {
        return Err(Error::RequestParamTypeError(key.to_owned()));
    }
    let mut vec: Vec<T> = Vec::new();
    for element in value.as_array().unwrap() {
        if !element.is_object() {
            return Err(Error::RequestParamTypeError(key.to_owned()));
        }
        vec.push(T::from_map(element.as_object().unwrap())?)
    }
    Ok(vec)
}
