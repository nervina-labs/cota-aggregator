use crate::schema::issuer_infos::dsl::issuer_infos;
use crate::schema::issuer_infos::{avatar, description, localization, lock_hash, name, version};
use crate::utils::error::Error;
use diesel::*;
use log::error;
use serde::{Deserialize, Serialize};

use super::get_conn;

#[derive(Serialize, Deserialize, Queryable, Debug, Clone, Eq, PartialEq, Default)]
pub struct IssuerInfoDb {
    pub version:      String,
    pub name:         String,
    pub avatar:       String,
    pub description:  String,
    pub localization: String,
}

pub fn get_issuer_info_by_lock_hash(lock_hash_: [u8; 32]) -> Result<Option<IssuerInfoDb>, Error> {
    let lock_hash_hex = hex::encode(lock_hash_);
    let issuers: Vec<IssuerInfoDb> = issuer_infos
        .select((version, name, avatar, description, localization))
        .filter(lock_hash.eq(lock_hash_hex))
        .limit(1)
        .load::<IssuerInfoDb>(&get_conn())
        .map_or_else(
            |e| {
                error!("Query issuer info error: {}", e.to_string());
                Err(Error::DatabaseQueryInvalid(e.to_string()))
            },
            Ok,
        )?;
    Ok(issuers.get(0).cloned())
}
