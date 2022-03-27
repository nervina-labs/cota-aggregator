use crate::models::claim::{get_claim_cota_by_lock_hash_with_conn, ClaimDb};
use crate::models::define::{
    get_define_cota_by_cota_id, get_define_cota_by_lock_hash_with_conn, DefineDb,
};
use crate::models::helper::establish_connection;
use crate::models::hold::{
    check_hold_cota_by_lock_hash, get_hold_cota_by_lock_hash_and_page,
    get_hold_cota_by_lock_hash_with_conn, HoldDb,
};
use crate::models::scripts::get_script_id_by_lock_script;
use crate::models::withdrawal::{
    get_sender_lock_by_script_id, get_withdrawal_cota_by_cota_ids,
    get_withdrawal_cota_by_lock_hash_with_conn, get_withdrawal_cota_by_script_id, WithdrawDb,
    WithdrawNFTDb,
};
use crate::models::DBTotalResult;
use crate::utils::error::Error;
use cota_smt::smt::blake2b_256;

type DBAllResult = Result<(Vec<DefineDb>, Vec<HoldDb>, Vec<WithdrawDb>, Vec<ClaimDb>), Error>;

pub fn get_all_cota_by_lock_hash(lock_hash: [u8; 32]) -> DBAllResult {
    let conn = &establish_connection();
    let defines = get_define_cota_by_lock_hash_with_conn(conn, lock_hash)?;
    let holds = get_hold_cota_by_lock_hash_with_conn(conn, lock_hash, None)?;
    let withdrawals = get_withdrawal_cota_by_lock_hash_with_conn(conn, lock_hash, None)?;
    let claims = get_claim_cota_by_lock_hash_with_conn(conn, lock_hash)?;
    Ok((defines.0, holds.0, withdrawals.0, claims.0))
}

pub fn get_hold_cota(lock_script: &[u8], page: i64, page_size: i64) -> DBTotalResult<HoldDb> {
    let lock_hash = blake2b_256(lock_script);
    get_hold_cota_by_lock_hash_and_page(lock_hash, page, page_size)
}

pub fn get_withdrawal_cota(
    lock_script: &[u8],
    page: i64,
    page_size: i64,
) -> DBTotalResult<WithdrawNFTDb> {
    let conn = &establish_connection();
    let script_id_opt = get_script_id_by_lock_script(conn, lock_script)?;
    match script_id_opt {
        Some(script_id) => get_withdrawal_cota_by_script_id(conn, script_id, page, page_size),
        None => Ok((vec![], 0, 0)),
    }
}

pub fn get_mint_cota(lock_script: &[u8], page: i64, page_size: i64) -> DBTotalResult<WithdrawDb> {
    let conn = &establish_connection();
    let lock_hash = blake2b_256(lock_script);
    let defines = get_define_cota_by_lock_hash_with_conn(conn, lock_hash)?.0;
    let cota_ids: Vec<[u8; 20]> = defines.into_iter().map(|define| define.cota_id).collect();
    get_withdrawal_cota_by_cota_ids(conn, lock_hash, cota_ids, page, page_size)
}

pub fn check_cota_claimed(
    lock_hash: [u8; 32],
    cota_id: [u8; 20],
    index: [u8; 4],
) -> Result<(bool, u64), Error> {
    check_hold_cota_by_lock_hash(lock_hash, (cota_id, index))
}

pub fn get_sender_lock_hash_by_cota_nft(
    lock_script: &[u8],
    cota_id: [u8; 20],
    token_index: [u8; 4],
) -> Result<Option<String>, Error> {
    let conn = &establish_connection();
    let lock_script_id_opt = get_script_id_by_lock_script(conn, lock_script)?;
    if lock_script_id_opt.is_none() {
        return Ok(None);
    }
    get_sender_lock_by_script_id(conn, lock_script_id_opt.unwrap(), cota_id, token_index)
}

pub fn get_define_info_by_cota_id(cota_id: [u8; 20]) -> Result<Option<(u32, u32, u8)>, Error> {
    let define_opt: Option<DefineDb> = get_define_cota_by_cota_id(cota_id)?;
    Ok(define_opt.map(|define: DefineDb| (define.total, define.issued, define.configure)))
}
