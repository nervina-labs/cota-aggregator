use crate::utils::error::Error;
use jsonrpc_http_server::jsonrpc_core::serde_json::{Map, Number};
use jsonrpc_http_server::jsonrpc_core::Value;

pub trait Inserter {
    fn insert_hex(&mut self, k: &str, v: &[u8]) -> Option<Value>;
    fn insert_i64(&mut self, k: &str, v: i64) -> Option<Value>;
    fn insert_u64(&mut self, k: &str, v: u64) -> Option<Value>;
    fn insert_u32(&mut self, k: &str, v: u32) -> Option<Value>;
    fn insert_u8(&mut self, k: &str, v: u8) -> Option<Value>;
    fn insert_str(&mut self, k: &str, v: String) -> Option<Value>;
    fn insert_array(&mut self, k: &str, v: Vec<Value>) -> Option<Value>;
    fn insert_null(&mut self, k: &str) -> Option<Value>;
    fn insert_obj(&mut self, k: &str, v: Map<String, Value>) -> Option<Value>;
    fn insert_obj_vec(&mut self, k: &str, v: Vec<Map<String, Value>>) -> Option<Value>;
}

impl Inserter for Map<String, Value> {
    fn insert_hex(&mut self, k: &str, v: &[u8]) -> Option<Value> {
        self.insert(
            k.to_string(),
            Value::String(format!("0x{}", hex::encode(v))),
        )
    }

    fn insert_i64(&mut self, k: &str, v: i64) -> Option<Value> {
        self.insert(k.to_string(), Value::Number(Number::from(v)))
    }

    fn insert_u64(&mut self, k: &str, v: u64) -> Option<Value> {
        self.insert(k.to_string(), Value::Number(Number::from(v)))
    }

    fn insert_u32(&mut self, k: &str, v: u32) -> Option<Value> {
        self.insert(k.to_string(), Value::Number(Number::from(v)))
    }

    fn insert_u8(&mut self, k: &str, v: u8) -> Option<Value> {
        self.insert(k.to_string(), Value::Number(Number::from(v)))
    }

    fn insert_str(&mut self, k: &str, v: String) -> Option<Value> {
        self.insert(k.to_string(), Value::String(v))
    }

    fn insert_array(&mut self, k: &str, v: Vec<Value>) -> Option<Value> {
        self.insert(k.to_string(), Value::Array(v))
    }

    fn insert_null(&mut self, k: &str) -> Option<Value> {
        self.insert(k.to_string(), Value::Null)
    }

    fn insert_obj(&mut self, k: &str, v: Map<String, Value>) -> Option<Value> {
        self.insert(k.to_string(), Value::Object(v))
    }

    fn insert_obj_vec(&mut self, k: &str, v: Vec<Map<String, Value>>) -> Option<Value> {
        let vec: Vec<Value> = v.into_iter().map(|v_| Value::Object(v_)).collect();
        self.insert(k.to_string(), Value::Array(vec))
    }
}

pub fn parse_json_err(_err: serde_json::Error) -> Error {
    Error::Other("Json parse error".to_string())
}
