use super::scripts::get_script_map_by_ids;
use crate::schema::register_cota_kv_pairs::dsl::register_cota_kv_pairs;
use crate::schema::register_cota_kv_pairs::*;
use crate::utils::error::Error;
use crate::POOL;
use diesel::*;
use log::error;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Queryable, Debug, Clone)]
pub struct AccountDb {
    pub cota_cell_id:   u64,
    pub lock_script_id: i64,
}
pub fn get_ccid_lock_script(
    lock_hash_opt: Option<[u8; 32]>,
    ccid: Option<u64>,
) -> Result<Option<(Vec<u8>, u64)>, Error> {
    if lock_hash_opt.is_none() && ccid.is_none() {
        return Err(Error::DatabaseQueryError(
            "lock hash and ccid cannot all be empty".to_string(),
        ));
    }
    let conn = &POOL.clone().get().expect("Mysql pool connection error");
    let accounts = match lock_hash_opt {
        Some(lock_hash_) => register_cota_kv_pairs
            .select((cota_cell_id, lock_script_id))
            .filter(lock_hash.eq(hex::encode(lock_hash_)))
            .limit(1)
            .load::<AccountDb>(conn)
            .map_err(|e| {
                error!("Query registry account error: {}", e.to_string());
                Error::DatabaseQueryError(e.to_string())
            })?,
        None => register_cota_kv_pairs
            .select((cota_cell_id, lock_script_id))
            .filter(cota_cell_id.eq(ccid.unwrap()))
            .limit(1)
            .load::<AccountDb>(conn)
            .map_err(|e| {
                error!("Query registry account error: {}", e.to_string());
                Error::DatabaseQueryError(e.to_string())
            })?,
    };
    match accounts.get(0).cloned() {
        Some(account) => {
            let script_id = account.lock_script_id;
            let script_map = get_script_map_by_ids(vec![script_id])?;
            let lock_ccid = script_map
                .get(&script_id)
                .cloned()
                .map(|lock| (lock, account.cota_cell_id));
            Ok(lock_ccid)
        }
        None => Ok(None),
    }
}
