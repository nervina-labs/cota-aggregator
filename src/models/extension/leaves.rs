use crate::{
    models::{
        block::get_syncer_tip_block_number,
        get_conn,
        helper::{parse_lock_hash, PAGE_SIZE},
        DBResult,
    },
    schema::extension_kv_pairs::dsl::extension_kv_pairs,
    schema::extension_kv_pairs::{key, lock_hash, lock_hash_crc, value},
    utils::{
        error::Error,
        helper::{diff_time, parse_bytes_n},
    },
};
use chrono::prelude::*;
use diesel::*;
use log::error;
use serde::Serialize;
use sparse_merkle_tree::H256;

#[derive(Serialize, Debug, Copy, Clone, Eq, PartialEq)]
pub struct ExtensionLeafDb {
    pub key:   [u8; 32],
    pub value: [u8; 32],
}

#[derive(Queryable, Debug, Clone, Eq, PartialEq)]
pub struct ExtensionLeaf {
    pub key:   String,
    pub value: String,
}

pub fn get_extension_leaves_by_lock_hash(lock_hash_: [u8; 32]) -> DBResult<ExtensionLeafDb> {
    let start_time = Local::now().timestamp_millis();
    let (lock_hash_hex, lock_hash_crc_) = parse_lock_hash(lock_hash_);
    let mut page: i64 = 0;
    let mut leaves: Vec<ExtensionLeafDb> = Vec::new();
    loop {
        let leaves_page = extension_kv_pairs
            .select((key, value))
            .filter(lock_hash_crc.eq(lock_hash_crc_))
            .filter(lock_hash.eq(lock_hash_hex.clone()))
            .limit(PAGE_SIZE)
            .offset(PAGE_SIZE * page)
            .load::<ExtensionLeaf>(&get_conn())
            .map_or_else(
                |e| {
                    error!("Query extension error: {}", e.to_string());
                    Err(Error::DatabaseQueryInvalid(e.to_string()))
                },
                |leaves_| Ok(parse_extension_leaves(leaves_)),
            )?;
        let length = leaves_page.len();
        leaves.extend(leaves_page);
        if length < (PAGE_SIZE as usize) {
            break;
        }
        page += 1;
    }
    let block_height = get_syncer_tip_block_number()?;
    diff_time(start_time, "SQL get_extension_leaves_by_lock_hash");
    Ok((leaves, block_height))
}

pub fn get_extension_leaf_by_lock_hash(
    lock_hash_: [u8; 32],
    key_: H256,
) -> Result<Option<ExtensionLeafDb>, Error> {
    let start_time = Local::now().timestamp_millis();
    let (lock_hash_hex, lock_hash_crc_) = parse_lock_hash(lock_hash_);
    let leaves: Vec<ExtensionLeafDb> = extension_kv_pairs
        .select((key, value))
        .filter(lock_hash_crc.eq(lock_hash_crc_))
        .filter(lock_hash.eq(lock_hash_hex))
        .filter(key.eq(hex::encode(key_.as_slice())))
        .limit(1)
        .load::<ExtensionLeaf>(&get_conn())
        .map_or_else(
            |e| {
                error!("Query extension error: {}", e.to_string());
                Err(Error::DatabaseQueryInvalid(e.to_string()))
            },
            |leaves_| Ok(parse_extension_leaves(leaves_)),
        )?;
    diff_time(start_time, "SQL get_extension_leaves_by_lock_hash");
    Ok(leaves.first().cloned())
}

pub fn parse_extension_leaves(leaves: Vec<ExtensionLeaf>) -> Vec<ExtensionLeafDb> {
    leaves
        .into_iter()
        .map(|leaf| ExtensionLeafDb {
            key:   parse_bytes_n::<32>(leaf.key).unwrap(),
            value: parse_bytes_n::<32>(leaf.value).unwrap(),
        })
        .collect()
}
