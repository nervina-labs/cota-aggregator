use crate::models::claim::{
    get_claim_cota_by_lock_hash, get_claim_cota_by_lock_hash_with_conn, ClaimDb,
};
use crate::models::define::{
    get_define_cota_by_lock_hash, get_define_cota_by_lock_hash_with_conn, DefineDb,
};
use crate::models::helper::establish_connection;
use crate::models::hold::{
    get_hold_cota_by_lock_hash, get_hold_cota_by_lock_hash_with_conn, HoldDb,
};
use crate::models::scripts::get_script_id_by_lock_script;
use crate::models::withdrawal::{
    get_withdrawal_cota_by_lock_hash, get_withdrawal_cota_by_lock_hash_with_conn,
    get_withdrawal_cota_by_script_id, WithdrawDb,
};
use crate::models::DBResult;
use crate::utils::error::Error;
use cota_smt::smt::blake2b_256;

type DBAllResult = Result<(Vec<DefineDb>, Vec<HoldDb>, Vec<WithdrawDb>, Vec<ClaimDb>), Error>;

pub fn get_all_cota_by_lock_hash(lock_hash: [u8; 32]) -> DBAllResult {
    let conn = &establish_connection();
    let defines = get_define_cota_by_lock_hash_with_conn(conn, lock_hash)?;
    let holds = get_hold_cota_by_lock_hash_with_conn(conn, lock_hash, None)?;
    let withdrawals = get_withdrawal_cota_by_lock_hash_with_conn(conn, lock_hash, None)?;
    let claims = get_claim_cota_by_lock_hash_with_conn(conn, lock_hash)?;
    Ok((defines, holds, withdrawals, claims))
}

pub fn get_hold_cota(lock_script: Vec<u8>) -> DBResult<HoldDb> {
    let lock_hash = blake2b_256(&lock_script);
    get_hold_cota_by_lock_hash(lock_hash, None)
}

pub fn get_withdrawal_cota(lock_script: Vec<u8>) -> DBResult<WithdrawDb> {
    let conn = &establish_connection();
    let script_id = get_script_id_by_lock_script(conn, &lock_script)?;
    let withdrawals = get_withdrawal_cota_by_script_id(conn, script_id, lock_script)?;
    Ok(withdrawals)
}

pub fn get_mint_cota(lock_script: Vec<u8>) -> DBResult<WithdrawDb> {
    let conn = &establish_connection();
    let lock_hash = blake2b_256(&lock_script);
    let defines = get_define_cota_by_lock_hash_with_conn(conn, lock_hash)?;
    let cota_ids: Vec<[u8; 20]> = defines.into_iter().map(|define| define.cota_id).collect();
    let script_id = get_script_id_by_lock_script(conn, &lock_script)?;
    let withdrawals = get_withdrawal_cota_by_script_id(conn, script_id, lock_script)?;
    Ok(withdrawals)
}
