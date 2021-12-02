use jsonrpc_http_server::jsonrpc_core::Value;

fn remove_0x(str: &str) -> &str {
    if str.contains("0x") {
        &str[2..]
    } else {
        str
    }
}

pub fn check_request_params(values: Vec<Value>) -> Option<Value> {
    for value in values.iter() {
        if !value.is_string() {
            return Some(Value::String(
                "Request parameters must be string".to_string(),
            ));
        }
        let hex_str = value.as_str().unwrap();
        if !hex_str.contains("0x") {
            return Some(Value::String(
                "Request parameters must be prefixed with 0x".to_string(),
            ));
        }
        let hex_str = remove_0x(hex_str);
        if hex_str.len() != 64 {
            return Some(Value::String(
                "Request parameters must be 32 bytes".to_string(),
            ));
        }
        if hex::decode(hex_str).is_err() {
            return Some(Value::String(
                "Request parameters must be hex string".to_string(),
            ));
        }
    }
    None
}

#[test]
fn test_check_request_params() {
    let values = vec![Value::String(
        "0xea28c98f38b4a57aa81756b167bb37fa42daf67edbc9863afb8172096ed301c2".to_string(),
    )];
    assert!(check_request_params(values).is_none());

    let values = vec![
        Value::String(
            "0xea28c98f38b4a57aa81756b167bb37fa42daf67edbc9863afb8172096ed301c2000000000088993355"
                .to_string(),
        ),
        Value::String("0x28c98f38b4a57aa81756b167bb37fa42daf67edbc9863afb8172096e".to_string()),
    ];
    assert!(check_request_params(values).unwrap() == "Request parameters must be 32 bytes");
}
