use crate::schema::check_infos::dsl::check_infos;
use crate::schema::check_infos::*;
use crate::utils::error::Error;
use diesel::*;
use log::error;

use super::get_conn;

pub fn get_syncer_tip_block_number() -> Result<u64, Error> {
    check_infos
        .select(block_number)
        .order(block_number.desc())
        .first::<u64>(&get_conn())
        .map_err(|e| {
            error!("Query block number error: {}", e.to_string());
            Error::DatabaseQueryError(e.to_string())
        })
}
