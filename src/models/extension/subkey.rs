use crate::{
    models::get_conn,
    schema::sub_key_kv_pairs::dsl::sub_key_kv_pairs,
    schema::sub_key_kv_pairs::{alg_index, ext_data, lock_hash, pubkey_hash},
    utils::{error::Error, helper::diff_time},
};
use chrono::prelude::*;
use diesel::*;
use log::error;
use serde::Serialize;

#[derive(Serialize, Debug, Clone, Eq, PartialEq)]
pub struct SubkeyDb {
    pub ext_data:  u32,
    pub alg_index: u16,
}

#[derive(Queryable, Debug, Clone, Eq, PartialEq)]
struct Subkey {
    pub ext_data:  u32,
    pub alg_index: u32,
}

pub fn get_subkey_by_pubkey_hash(
    lock_hash_: [u8; 32],
    pubkey_hash_: [u8; 20],
) -> Result<Option<SubkeyDb>, Error> {
    let start_time = Local::now().timestamp_millis();
    let pubkey_hash_str = hex::encode(pubkey_hash_);
    let lock_hash_str = hex::encode(lock_hash_);
    let subkeys: Vec<SubkeyDb> = sub_key_kv_pairs
        .select((ext_data, alg_index))
        .filter(pubkey_hash.eq(pubkey_hash_str))
        .filter(lock_hash.eq(lock_hash_str))
        .limit(1)
        .load::<Subkey>(&get_conn())
        .map_or_else(
            |e| {
                error!("Query subkey error: {}", e.to_string());
                Err(Error::DatabaseQueryError(e.to_string()))
            },
            |subkeys_| Ok(parse_subkey(subkeys_)),
        )?;
    diff_time(start_time, "SQL get_subkey_by_pubkey_hash");
    Ok(subkeys.first().cloned())
}

fn parse_subkey(subkeys: Vec<Subkey>) -> Vec<SubkeyDb> {
    subkeys
        .into_iter()
        .map(|subkey| SubkeyDb {
            ext_data:  subkey.ext_data,
            alg_index: subkey.alg_index as u16,
        })
        .collect()
}
