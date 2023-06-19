use std::collections::HashMap;

use crate::schema::check_infos::dsl::check_infos;
use crate::schema::check_infos::*;
use crate::utils::error::Error;
use diesel::*;
use log::error;

use super::get_conn;

pub fn get_syncer_tip_block_number() -> Result<u64, Error> {
    let (tip_block_number, _) = get_syncer_tip_block_numbers()?;
    Ok(tip_block_number)
}

#[derive(Queryable)]
pub struct BlockDb {
    pub block_number: u64,
    pub check_type:   u8,
}
const BLOCK_CHECK_TYPE: u8 = 0;
const METADATA_CHECK_TYPE: u8 = 1;
pub fn get_syncer_tip_block_numbers() -> Result<(u64, u64), Error> {
    let blocks = check_infos
        .select((block_number, check_type))
        .order(block_number.desc())
        .limit(4)
        .load::<BlockDb>(&get_conn())
        .map_err(|e| {
            error!("Query block number error: {}", e.to_string());
            Error::DatabaseQueryError(e.to_string())
        })?;
    let mut map: HashMap<u8, u64> = HashMap::with_capacity(2);
    for block in blocks.iter() {
        if map.get(&block.check_type).is_none() {
            map.insert(block.check_type, block.block_number);
        }
    }
    let tip_block_number = map.get(&BLOCK_CHECK_TYPE).unwrap_or(&0);
    let tip_metadata_number = map.get(&METADATA_CHECK_TYPE).unwrap_or(&0);
    Ok((*tip_block_number, *tip_metadata_number))
}
