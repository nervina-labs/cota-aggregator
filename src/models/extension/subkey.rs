use super::leaves::{parse_extension_leaves, ExtensionLeaf, ExtensionLeafDb};
use crate::{
    models::get_conn,
    schema::extension_kv_pairs::dsl::extension_kv_pairs,
    schema::extension_kv_pairs::{key, value},
    utils::{error::Error, helper::diff_time},
};
use chrono::prelude::*;
use diesel::*;
use log::error;

pub fn get_subkey_leaf_by_pubkey_hash(
    pubkey_hash: [u8; 20],
) -> Result<Option<ExtensionLeafDb>, Error> {
    let start_time = Local::now().timestamp_millis();
    let pubkey_hash_str = hex::encode(pubkey_hash);
    let sub_type = hex::encode("subkey".as_bytes());
    let leaves: Vec<ExtensionLeafDb> = extension_kv_pairs
        .select((key, value))
        .filter(value.like(format!("%{}%", pubkey_hash_str)))
        .filter(key.like(format!("%{}%", sub_type)))
        .limit(1)
        .load::<ExtensionLeaf>(&get_conn())
        .map_or_else(
            |e| {
                error!("Query extension error: {}", e.to_string());
                Err(Error::DatabaseQueryError(e.to_string()))
            },
            |leaves_| Ok(parse_extension_leaves(leaves_)),
        )?;
    diff_time(start_time, "SQL get_subkey_leaf_by_pubkey_hash");
    Ok(leaves.first().cloned())
}
