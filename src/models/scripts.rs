use super::helper::SqlConnection;
use crate::schema::scripts::dsl::scripts;
use crate::schema::scripts::*;
use crate::schema::scripts::{args, code_hash, hash_type};
use crate::utils::error::Error;
use crate::utils::helper::{parse_bytes, parse_bytes_n};
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

pub fn get_script_map_by_ids(
    conn: &SqlConnection,
    script_ids: Vec<i64>,
) -> Result<HashMap<i64, Vec<u8>>, Error> {
    let scripts_db = scripts
        .select((id, code_hash, hash_type, args))
        .filter(id.eq_any(script_ids))
        .load::<Script>(conn)
        .map_or_else(
            |e| {
                error!("Query script error: {}", e.to_string());
                Err(Error::DatabaseQueryError(e.to_string()))
            },
            |scripts_| Ok(parse_script(scripts_)),
        )?;
    let scripts_: Vec<(i64, Vec<u8>)> = scripts_db
        .iter()
        .map(|script_db| (script_db.id, generate_script_vec(script_db)))
        .collect();
    let script_map: HashMap<i64, Vec<u8>> = scripts_.into_iter().collect();
    Ok(script_map)
}

pub fn get_script_id_by_lock_script(
    conn: &SqlConnection,
    lock_script: &[u8],
) -> Result<i64, Error> {
    let lock = LockScript::from_slice(lock_script).unwrap();
    let lock_code_hash = hex::encode(lock.code_hash().as_slice().to_vec());
    let lock_args = hex::encode(lock.args().raw_data().to_vec());
    let script_ids: Vec<i64> = scripts
        .select(id)
        .filter(code_hash.eq(lock_code_hash))
        .filter(hash_type.eq(lock.hash_type().as_slice()[0]))
        .filter(args.eq(lock_args))
        .load::<i64>(conn)
        .map_err(|e| {
            error!("Query script error: {}", e.to_string());
            Error::DatabaseQueryError(e.to_string())
        })?;
    script_ids.get(0).map_or_else(
        || Err(Error::DatabaseQueryEmpty("script".to_string())),
        |id_| Ok(*id_),
    )
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
