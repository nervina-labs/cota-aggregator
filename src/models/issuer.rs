use super::helper::SqlConnection;
use crate::schema::issuer_infos::dsl::issuer_infos;
use crate::schema::issuer_infos::{avatar, description, lock_hash, name};
use crate::utils::error::Error;
use diesel::*;
use log::error;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Queryable, Debug, Clone, Eq, PartialEq)]
pub struct IssuerInfoDb {
    pub name:        String,
    pub avatar:      String,
    pub description: String,
}

pub fn get_issuer_info_by_lock_hash(
    conn: &SqlConnection,
    lock_hash_: [u8; 32],
) -> Result<Option<IssuerInfoDb>, Error> {
    let lock_hash_hex = hex::encode(lock_hash_);
    let issuers: Vec<IssuerInfoDb> = issuer_infos
        .select((name, avatar, description))
        .filter(lock_hash.eq(lock_hash_hex))
        .load::<IssuerInfoDb>(conn)
        .map_or_else(
            |e| {
                error!("Query issuer info error: {}", e.to_string());
                Err(Error::DatabaseQueryError(e.to_string()))
            },
            |issuers_| Ok(issuers_),
        )?;
    Ok(issuers.get(0).cloned())
}
