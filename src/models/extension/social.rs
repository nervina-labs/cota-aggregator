use crate::{
    models::{get_conn, helper::parse_lock_hash},
    schema::social_kv_pairs::dsl::social_kv_pairs,
    schema::social_kv_pairs::{lock_hash, lock_hash_crc, must, recovery_mode, signers, total},
    utils::{error::Error, helper::diff_time},
};
use chrono::prelude::*;
use diesel::*;
use log::error;
use serde::Serialize;

#[derive(Serialize, Debug, Clone, Eq, PartialEq)]
pub struct SocialRecoveryDb {
    pub recovery_mode: u8,
    pub must:          u8,
    pub total:         u8,
    pub signers:       Vec<Vec<u8>>,
}

#[derive(Queryable, Debug, Clone, Eq, PartialEq)]
pub struct SocialRecovery {
    pub recovery_mode: u8,
    pub must:          u8,
    pub total:         u8,
    pub signers:       String,
}

pub fn get_social_config_by_lock(lock_hash_: [u8; 32]) -> Result<Option<SocialRecoveryDb>, Error> {
    let start_time = Local::now().timestamp_millis();
    let (lock_hash_hex, lock_hash_crc_) = parse_lock_hash(lock_hash_);
    let socials = social_kv_pairs
        .select((recovery_mode, must, total, signers))
        .filter(lock_hash_crc.eq(lock_hash_crc_))
        .filter(lock_hash.eq(lock_hash_hex))
        .limit(1)
        .load::<SocialRecovery>(&get_conn())
        .map_or_else(
            |e| {
                error!("Query social error: {}", e.to_string());
                Err(Error::DatabaseQueryInvalid(e.to_string()))
            },
            |socials_| Ok(parse_social(socials_)),
        )?;
    diff_time(start_time, "SQL get_social_config_by_lock");
    Ok(socials.first().cloned())
}

fn parse_social(socials: Vec<SocialRecovery>) -> Vec<SocialRecoveryDb> {
    socials
        .into_iter()
        .map(|social| SocialRecoveryDb {
            recovery_mode: social.recovery_mode,
            must:          social.must,
            total:         social.total,
            signers:       parse_signers(social.signers),
        })
        .collect()
}

fn parse_signers(signers_str: String) -> Vec<Vec<u8>> {
    signers_str
        .split(',')
        .map(|str| hex::decode(str).unwrap())
        .collect()
}
