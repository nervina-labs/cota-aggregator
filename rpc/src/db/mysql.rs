use crate::config::load_config;
use crate::db::types::{ClaimDb, DefineDb, HoldDb, ScriptDb, WithdrawDb, WithdrawWithScriptIdDb};
use crate::error::Error;
use crate::utils::{generate_crc, parse_bytes, parse_bytes_n};
use cota_smt::ckb_types::packed::{Byte32, BytesBuilder, ScriptBuilder};
use cota_smt::ckb_types::prelude::*;
use cota_smt::molecule::prelude::Byte;
use lazy_static::lazy_static;
use log::error;
use mysql::prelude::*;
use mysql::*;
use std::collections::HashMap;
use std::sync::MutexGuard;
use std::{result::Result, sync::Mutex};

lazy_static! {
    pub static ref CONN: Mutex<PooledConn> = {
        let url = load_config().database_url;
        let max: usize = if let Ok(max_pool) = env::var("MAX_POOL") {
            max_pool.parse::<usize>().unwrap()
        } else {
            20
        };
        let pool = Pool::new_manual(10usize, max, Opts::from_url(&url).unwrap()).expect("Database pool error");
        Mutex::new(pool.get_conn().expect("Database connection error"))
    };
}

pub fn get_define_cota_by_lock_hash(lock_hash: [u8; 32]) -> Result<Vec<DefineDb>, Error> {
    let (lock_hash_hex, lock_hash_crc) = parse_lock_hash(lock_hash);
    CONN.lock()
        .unwrap()
        .query_map(
            format!(
                "select cota_id, total, issued, configure from define_cota_nft_kv_pairs \
                where lock_hash_crc = '{}' and lock_hash = '{}'",
                lock_hash_crc, lock_hash_hex
            ),
            |(cota_id, total, issued, configure)| DefineDb {
                cota_id:   parse_mysql_bytes_n::<20>(cota_id),
                total:     from_value::<u32>(total),
                issued:    from_value::<u32>(issued),
                configure: from_value::<u8>(configure),
            },
        )
        .map_err(|e| {
            error!("Query define error: {}", e.to_string());
            Error::DatabaseQueryError(e.to_string())
        })
}

pub fn get_define_cota_by_lock_hash_and_cota_id(
    lock_hash: [u8; 32],
    cota_id: [u8; 20],
) -> Result<Option<DefineDb>, Error> {
    let (lock_hash_hex, lock_hash_crc) = parse_lock_hash(lock_hash);
    let cota_id_hex = hex::encode(cota_id);

    let result = CONN
        .lock()
        .unwrap()
        .query_map(
            format!(
                "select total, issued, configure from define_cota_nft_kv_pairs \
                 where lock_hash_crc = '{}' and lock_hash = '{}' and cota_id = '{}'",
                lock_hash_crc, lock_hash_hex, cota_id_hex
            ),
            |(total, issued, configure): (Value, Value, Value)| DefineDb {
                cota_id,
                total: from_value::<u32>(total),
                issued: from_value::<u32>(issued),
                configure: from_value::<u8>(configure),
            },
        )
        .map_err(|e| {
            error!("Query define error: {}", e.to_string());
            Error::DatabaseQueryError(e.to_string())
        })?;
    Ok(result.get(0).map(|v| *v))
}

pub fn get_hold_cota_by_lock_hash(
    lock_hash: [u8; 32],
    cota_id_and_token_index_pairs: Option<Vec<([u8; 20], [u8; 4])>>,
) -> Result<Vec<HoldDb>, Error> {
    let (lock_hash_hex, lock_hash_crc) = parse_lock_hash(lock_hash);
    let statement = match cota_id_and_token_index_pairs {
        Some(pairs) => {
            let (cota_id_array, token_index_array) = parse_cota_id_and_token_index_pairs(pairs);
            format!(
                "select cota_id, token_index, configure, state, characteristic from hold_cota_nft_kv_pairs where \
                 lock_hash_crc = '{}' and lock_hash = '{}' and cota_id in ('{}') and token_index in ({})",
                lock_hash_crc, lock_hash_hex, cota_id_array, token_index_array
            )
        }
        None => format!(
            "select cota_id, token_index, configure, state, characteristic from hold_cota_nft_kv_pairs \
            where lock_hash_crc = '{}' and lock_hash = '{}'", lock_hash_crc, lock_hash_hex
        ),
    };
    CONN.lock()
        .unwrap()
        .query_map(
            statement,
            |(cota_id, token_index, configure, state, characteristic)| HoldDb {
                cota_id:        parse_mysql_bytes_n::<20>(cota_id),
                token_index:    from_value::<u32>(token_index).to_be_bytes(),
                configure:      from_value::<u8>(configure),
                state:          from_value::<u8>(state),
                characteristic: parse_mysql_bytes_n::<20>(characteristic),
            },
        )
        .map_err(|e| {
            error!("Query hold error: {}", e.to_string());
            Error::DatabaseQueryError(e.to_string())
        })
}

pub fn get_withdrawal_cota_by_lock_hash(
    lock_hash: [u8; 32],
    cota_id_and_token_index_pairs: Option<Vec<([u8; 20], [u8; 4])>>,
) -> Result<Vec<WithdrawDb>, Error> {
    let (lock_hash_hex, lock_hash_crc) = parse_lock_hash(lock_hash);

    let statement = match cota_id_and_token_index_pairs {
        Some(pairs) =>  {
            let (cota_id_array, token_index_array) = parse_cota_id_and_token_index_pairs(pairs);
            format!("select cota_id, token_index, configure, state, characteristic, receiver_lock_script_id, out_point \
                    from withdraw_cota_nft_kv_pairs where lock_hash_crc = '{}' and lock_hash = '{}' and cota_id in ('{}') \
                    and token_index in ({})", lock_hash_crc, lock_hash_hex, cota_id_array, token_index_array)
        }
        None => format!("select cota_id, token_index, configure, state, characteristic, receiver_lock_script_id, out_point \
                        from withdraw_cota_nft_kv_pairs where lock_hash_crc = '{}' and lock_hash = '{}'", lock_hash_crc, lock_hash_hex),
    };

    let mut conn = CONN.lock().expect("Database connection error");
    let withdrawals_db = conn
        .query_map(
            statement,
            |(
                cota_id,
                token_index,
                configure,
                state,
                characteristic,
                receiver_lock_script_id,
                out_point,
            )| WithdrawWithScriptIdDb {
                cota_id:                 parse_mysql_bytes_n::<20>(cota_id),
                token_index:             from_value::<u32>(token_index).to_be_bytes(),
                configure:               from_value::<u8>(configure),
                state:                   from_value::<u8>(state),
                characteristic:          parse_mysql_bytes_n::<20>(characteristic),
                receiver_lock_script_id: from_value::<u64>(receiver_lock_script_id),
                out_point:               parse_mysql_bytes_n::<24>(out_point),
            },
        )
        .map_err(|e| {
            error!("Query withdrawal error: {}", e.to_string());
            Error::DatabaseQueryError(e.to_string())
        })?;
    if withdrawals_db.is_empty() {
        return Ok(vec![]);
    }
    let receiver_lock_script_ids: Vec<String> = withdrawals_db
        .iter()
        .map(|withdrawal| withdrawal.receiver_lock_script_id.to_string())
        .collect();
    let mut withdraw_db_vec: Vec<WithdrawDb> = vec![];
    let script_map = get_script_map_by_ids(conn, receiver_lock_script_ids)?;
    for withdrawal in withdrawals_db {
        let lock_script = script_map
            .get(&withdrawal.receiver_lock_script_id.to_string())
            .ok_or(Error::DatabaseQueryError("scripts".to_owned()))?
            .clone();
        withdraw_db_vec.push(WithdrawDb {
            cota_id:              withdrawal.cota_id,
            token_index:          withdrawal.token_index,
            configure:            withdrawal.configure,
            state:                withdrawal.state,
            characteristic:       withdrawal.characteristic,
            receiver_lock_script: lock_script,
            out_point:            withdrawal.out_point,
        })
    }
    Ok(withdraw_db_vec)
}

fn get_script_map_by_ids(
    mut conn: MutexGuard<PooledConn>,
    script_ids: Vec<String>,
) -> Result<HashMap<String, Vec<u8>>, Error> {
    let script_id_array = script_ids.join(",");
    let scripts_db = conn
        .query_map(
            format!(
                "select id, code_hash, hash_type, args from scripts where id in ({})",
                script_id_array
            ),
            |(id, code_hash, hash_type, args): (Value, Value, Value, Value)| ScriptDb {
                id:        from_value::<u64>(id),
                code_hash: parse_mysql_bytes_n::<32>(code_hash),
                hash_type: from_value::<u8>(hash_type),
                args:      parse_mysql_bytes_value(args),
            },
        )
        .map_err(|e| {
            error!("Query scripts error: {}", e.to_string());
            Error::DatabaseQueryError(e.to_string())
        })?;

    let scripts: Vec<(String, Vec<u8>)> = scripts_db
        .iter()
        .map(|script_db| (script_db.id.to_string(), generate_script_vec(script_db)))
        .collect();
    let script_map: HashMap<String, Vec<u8>> = scripts.into_iter().collect();
    Ok(script_map)
}

pub fn get_claim_cota_by_lock_hash(lock_hash: [u8; 32]) -> Result<Vec<ClaimDb>, Error> {
    let (lock_hash_hex, lock_hash_crc) = parse_lock_hash(lock_hash);
    CONN.lock()
        .unwrap()
        .query_map(
            format!(
                "select cota_id, token_index, out_point from claimed_cota_nft_kv_pairs \
                                where lock_hash_crc = '{}' and lock_hash = '{}'",
                lock_hash_crc, lock_hash_hex
            ),
            |(cota_id, token_index, out_point): (Value, Value, Value)| ClaimDb {
                cota_id:     parse_mysql_bytes_n::<20>(cota_id),
                token_index: from_value::<u32>(token_index).to_be_bytes(),
                out_point:   parse_mysql_bytes_n::<24>(out_point),
            },
        )
        .map_err(|e| {
            error!("Query claim error: {}", e.to_string());
            Error::DatabaseQueryError(e.to_string())
        })
}

pub fn get_all_cota_by_lock_hash(
    lock_hash: [u8; 32],
) -> Result<(Vec<DefineDb>, Vec<HoldDb>, Vec<WithdrawDb>, Vec<ClaimDb>), Error> {
    let defines = get_define_cota_by_lock_hash(lock_hash)?;
    let holds = get_hold_cota_by_lock_hash(lock_hash, None)?;
    let withdrawals = get_withdrawal_cota_by_lock_hash(lock_hash, None)?;
    let claims = get_claim_cota_by_lock_hash(lock_hash)?;
    Ok((defines, holds, withdrawals, claims))
}

fn parse_lock_hash(lock_hash: [u8; 32]) -> (String, u32) {
    (
        hex::encode(lock_hash),
        generate_crc(hex::encode(lock_hash).as_bytes()),
    )
}

fn parse_mysql_bytes_n<const N: usize>(v: Value) -> [u8; N] {
    let vec = from_value::<Vec<u8>>(v);
    parse_bytes_n::<N>(String::from_utf8(vec).unwrap()).unwrap()
}

fn parse_mysql_bytes_value(v: Value) -> Vec<u8> {
    let vec = from_value::<Vec<u8>>(v);
    parse_bytes(String::from_utf8(vec).unwrap()).unwrap()
}

fn parse_cota_id_and_token_index_pairs(pairs: Vec<([u8; 20], [u8; 4])>) -> (String, String) {
    let cota_id_hexes: Vec<String> = pairs.iter().map(|pair| hex::encode(pair.0)).collect();
    let token_index_hexes: Vec<String> = pairs
        .iter()
        .map(|pair| u32::from_be_bytes(pair.1).to_string())
        .collect();
    (cota_id_hexes.join("','"), token_index_hexes.join(","))
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
