use crate::models::helper::{establish_connection, SqlConnection};
use crate::schema::check_infos::dsl::check_infos;
use crate::schema::check_infos::*;
use crate::utils::error::Error;
use diesel::*;
use log::error;

pub fn get_syncer_tip_block_number_with_conn(conn: &SqlConnection) -> Result<u64, Error> {
    check_infos
        .select(block_number)
        .first::<u64>(conn)
        .map_err(|e| {
            error!("Query block number error: {}", e.to_string());
            Error::DatabaseQueryError(e.to_string())
        })
}

pub fn get_syncer_tip_block_number() -> Result<u64, Error> {
    get_syncer_tip_block_number_with_conn(&establish_connection())
}
