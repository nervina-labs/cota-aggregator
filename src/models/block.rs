use crate::schema::check_infos::dsl::check_infos;
use crate::schema::check_infos::*;
use crate::utils::error::Error;
use crate::POOL;
use diesel::*;
use log::error;

pub fn get_syncer_tip_block_number() -> Result<u64, Error> {
    let conn = &POOL.clone().get().expect("Mysql pool connection error");
    check_infos
        .select(block_number)
        .first::<u64>(conn)
        .map_err(|e| {
            error!("Query block number error: {}", e.to_string());
            Error::DatabaseQueryError(e.to_string())
        })
}
