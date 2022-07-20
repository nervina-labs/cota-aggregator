use crate::models::common::SECP256K1_BATCH_HASH_TYPE;
use crate::models::helper::{generate_crc, get_secp256k1_batch_code_hash, PAGE_SIZE};
use crate::schema::scripts::dsl::scripts;
use crate::schema::scripts::*;
use crate::schema::scripts::{args, code_hash, hash_type};
use crate::utils::error::Error;
use crate::utils::helper::{diff_time, parse_bytes, parse_bytes_n, parse_vec_n};
use crate::POOL;
use chrono::prelude::*;
use cota_smt::ckb_types::packed::{Byte32, BytesBuilder, Script as LockScript, ScriptBuilder};
use cota_smt::ckb_types::prelude::*;
use cota_smt::molecule::prelude::Byte;
use diesel::*;
use log::error;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Queryable, Debug)]
pub struct Script {
    pub id:        i64,
    pub code_hash: String,
    pub hash_type: u8,
    pub args:      String,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ScriptDb {
    pub id:        i64,
    pub code_hash: [u8; 32],
    pub hash_type: u8,
    pub args:      Vec<u8>,
}

pub fn get_script_map_by_ids(script_ids: Vec<i64>) -> Result<HashMap<i64, Vec<u8>>, Error> {
    let start_time = Local::now().timestamp_millis();
    let conn = &POOL.clone().get().expect("Mysql pool connection error");
    let mut scripts_dbs: Vec<ScriptDb> = vec![];
    let script_ids_subs: Vec<&[i64]> = script_ids.chunks(PAGE_SIZE as usize).collect();
    for script_ids_sub in script_ids_subs.into_iter() {
        let scripts_db = scripts
            .select((id, code_hash, hash_type, args))
            .filter(id.eq_any(script_ids_sub))
            .load::<Script>(conn)
            .map_or_else(
                |e| {
                    error!("Query script error: {}", e.to_string());
                    Err(Error::DatabaseQueryError(e.to_string()))
                },
                |scripts_| Ok(parse_script(scripts_)),
            )?;
        scripts_dbs.extend(scripts_db);
    }
    let scripts_: Vec<(i64, Vec<u8>)> = scripts_dbs
        .iter()
        .map(|script_db| (script_db.id, generate_script_vec(script_db)))
        .collect();
    let script_map: HashMap<i64, Vec<u8>> = scripts_.into_iter().collect();
    diff_time(start_time, "SQL get_script_map_by_ids");
    Ok(script_map)
}

pub fn get_script_id_by_lock_script(lock_script: &[u8]) -> Result<Option<i64>, Error> {
    let start_time = Local::now().timestamp_millis();
    let conn = &POOL.clone().get().expect("Mysql pool connection error");
    let lock = LockScript::from_slice(lock_script).unwrap();

    let lock_code_hash = hex::encode(lock.code_hash().as_slice());
    let lock_code_hash_crc = generate_crc(lock_code_hash.as_bytes());

    let lock_args = hex::encode(lock.args().raw_data().to_vec());
    let lock_args_crc = generate_crc(lock_args.as_bytes());

    let script_ids: Vec<i64> = scripts
        .select(id)
        .filter(code_hash_crc.eq(lock_code_hash_crc))
        .filter(hash_type.eq(lock.hash_type().as_slice()[0]))
        .filter(args_crc.eq(lock_args_crc))
        .filter(code_hash.eq(lock_code_hash))
        .filter(args.eq(lock_args))
        .limit(1)
        .load::<i64>(conn)
        .map_err(|e| {
            error!("Query script error: {}", e.to_string());
            Error::DatabaseQueryError(e.to_string())
        })?;
    diff_time(start_time, "SQL get_script_id_by_lock_script");
    Ok(script_ids.get(0).cloned())
}

pub fn get_secp256k1_batch_lock_hashes(master_args: [u8; 20]) -> Result<Vec<[u8; 32]>, Error> {
    let start_time = Local::now().timestamp_millis();
    let conn = &POOL.clone().get().expect("Mysql pool connection error");
    let lock_code_hash = get_secp256k1_batch_code_hash();
    let lock_code_hash_crc = generate_crc(lock_code_hash.as_bytes());
    let lock_hash_type = SECP256K1_BATCH_HASH_TYPE;
    let lock_master_args = hex::encode(master_args);

    let args_vec: Vec<String> = scripts
        .select(args)
        .filter(code_hash_crc.eq(lock_code_hash_crc))
        .filter(hash_type.eq(lock_hash_type))
        .filter(args.ge(lock_master_args))
        .filter(code_hash.eq(lock_code_hash.clone()))
        .load::<String>(conn)
        .map_err(|e| {
            error!("Query script error: {}", e.to_string());
            Error::DatabaseQueryError(e.to_string())
        })?;
    let lock_hashes: Vec<[u8; 32]> = args_vec
        .into_iter()
        .map(|lock_args| {
            let args_bytes: Vec<Byte> = hex::decode(lock_args)
                .unwrap()
                .into_iter()
                .map(|v| Byte::from_slice(&[v]).unwrap())
                .collect();
            let script = ScriptBuilder::default()
                .code_hash(
                    Byte32::from_slice(&hex::decode(lock_code_hash.clone()).unwrap()).unwrap(),
                )
                .hash_type(Byte::from(lock_hash_type))
                .args(BytesBuilder::default().set(args_bytes).build())
                .build();
            parse_vec_n::<32>(script.calc_script_hash().as_slice().to_vec())
        })
        .collect();
    diff_time(start_time, "SQL get_secp256k1_batch_lock_script_ids");
    Ok(lock_hashes)
}

fn parse_script(scripts_: Vec<Script>) -> Vec<ScriptDb> {
    scripts_
        .into_iter()
        .map(|script| ScriptDb {
            id:        script.id,
            code_hash: parse_bytes_n::<32>(script.code_hash).unwrap(),
            hash_type: script.hash_type,
            args:      parse_bytes(script.args).unwrap(),
        })
        .collect()
}

fn generate_script_vec(script_db: &ScriptDb) -> Vec<u8> {
    let args_bytes: Vec<Byte> = script_db.args.iter().map(|v| Byte::from(*v)).collect();
    let script = ScriptBuilder::default()
        .code_hash(Byte32::from_slice(&script_db.code_hash[..]).unwrap())
        .hash_type(Byte::from(script_db.hash_type))
        .args(BytesBuilder::default().set(args_bytes).build())
        .build();
    script.as_slice().to_vec()
}
