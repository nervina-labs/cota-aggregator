use super::get_conn;
use crate::schema::check_infos::dsl::check_infos;
use crate::schema::check_infos::*;
use crate::utils::error::Error;
use diesel::*;
use log::error;

const BLOCK_CHECK_TYPE: u8 = 0;
pub fn get_syncer_tip_block_number() -> Result<u64, Error> {
    check_infos
        .select(block_number)
        .filter(check_type.eq(BLOCK_CHECK_TYPE))
        .order(block_number.desc())
        .first::<u64>(&get_conn())
        .map_err(|e| {
            error!("Query tip block number error: {}", e.to_string());
            Error::DatabaseQueryError(e.to_string())
        })
}

const METADATA_CHECK_TYPE: u8 = 1;
pub fn get_syncer_tip_block_numbers() -> Result<(u64, u64), Error> {
    let tip_block_number = get_syncer_tip_block_number()?;
    let tip_metadata_number = check_infos
        .select(block_number)
        .filter(check_type.eq(METADATA_CHECK_TYPE))
        .order(block_number.desc())
        .first::<u64>(&get_conn())
        .map_err(|e| {
            error!("Query tip metadata number error: {}", e.to_string());
            Error::DatabaseQueryError(e.to_string())
        })?;
    Ok((tip_block_number, tip_metadata_number))
}
