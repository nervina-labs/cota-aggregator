use crate::models::class::ClassInfoDb;
use crate::models::hold::HoldDb;
use crate::response::helper::Inserter;
use jsonrpc_http_server::jsonrpc_core::serde_json::Map;
use jsonrpc_http_server::jsonrpc_core::Value;

pub fn parse_hold_response(
    holds: Vec<(HoldDb, Option<ClassInfoDb>)>,
    total: i64,
    page_size: i64,
    block_number: u64,
) -> Map<String, Value> {
    let nfts: Vec<Value> = holds.into_iter().map(parse_hold_value).collect();
    let mut map = Map::new();
    map.insert_i64("total", total);
    map.insert_i64("page_size", page_size);
    map.insert_u64("block_number", block_number);
    map.insert_array("nfts", nfts);
    map
}

fn parse_hold_value((hold, class_info): (HoldDb, Option<ClassInfoDb>)) -> Value {
    let mut map = Map::new();
    map.insert_hex("cota_id", &hold.cota_id);
    map.insert_hex("token_index", &hold.token_index);
    map.insert_hex("state", &[hold.state]);
    map.insert_hex("configure", &[hold.configure]);
    map.insert_hex("characteristic", &hold.characteristic);
    match class_info {
        Some(class) => {
            map.insert_str("name", class.name);
            map.insert_str("description", class.description);
            map.insert_str("image", class.image);
            map.insert_str("audio", class.audio);
            map.insert_str("video", class.video);
            map.insert_str("model", class.model);
            map.insert_str("meta_characteristic", class.characteristic);
            map.insert_str("properties", class.properties);
        }
        None => {
            map.insert_null("name");
            map.insert_null("description");
            map.insert_null("image");
            map.insert_null("audio");
            map.insert_null("video");
            map.insert_null("model");
            map.insert_null("meta_characteristic");
            map.insert_null("properties");
        }
    }
    Value::Object(map)
}
