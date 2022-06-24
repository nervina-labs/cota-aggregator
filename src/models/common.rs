use crate::models::claim::{get_claim_cota_by_lock_hash, is_exist_in_claim, ClaimDb};
use crate::models::class::{get_class_info_by_cota_id, ClassInfoDb};
use crate::models::define::{get_define_cota_by_cota_id, get_define_cota_by_lock_hash, DefineDb};
use crate::models::hold::{
    check_hold_cota_by_lock_hash, get_hold_cota_by_lock_hash, get_hold_cota_by_lock_hash_and_page,
    get_hold_cota_count_by_lock_hash, HoldDb,
};
use crate::models::scripts::get_script_id_by_lock_script;
use crate::models::withdrawal::{
    get_sender_lock_by_script_id, get_withdrawal_cota_by_cota_ids,
    get_withdrawal_cota_by_lock_hash, get_withdrawal_cota_by_script_id, WithdrawDb, WithdrawNFTDb,
};
use crate::models::DBTotalResult;
use crate::utils::error::Error;
use cota_smt::smt::blake2b_256;
use log::debug;

type DBAllResult = Result<(Vec<DefineDb>, Vec<HoldDb>, Vec<WithdrawDb>, Vec<ClaimDb>), Error>;

pub fn get_all_cota_by_lock_hash(lock_hash: [u8; 32]) -> DBAllResult {
    let defines = get_define_cota_by_lock_hash(lock_hash)?;
    let holds = get_hold_cota_by_lock_hash(lock_hash, &vec![])?;
    let withdrawals = get_withdrawal_cota_by_lock_hash(lock_hash, &vec![])?;
    let claims = get_claim_cota_by_lock_hash(lock_hash)?;
    Ok((defines.0, holds.0, withdrawals.0, claims.0))
}

pub fn get_hold_cota(
    lock_script: &[u8],
    page: i64,
    page_size: i64,
    cota_id_opt: Option<[u8; 20]>,
) -> DBTotalResult<(HoldDb, Option<ClassInfoDb>)> {
    let lock_hash = blake2b_256(lock_script);
    let (hold_nfts, total, block_height) =
        get_hold_cota_by_lock_hash_and_page(lock_hash, page, page_size, cota_id_opt)?;
    let mut nfts: Vec<(HoldDb, Option<ClassInfoDb>)> = vec![];
    for hold in hold_nfts {
        let class_info = get_class_info_by_cota_id(hold.cota_id)?;
        nfts.push((hold, class_info))
    }
    Ok((nfts, total, block_height))
}

pub fn get_withdrawal_cota(
    lock_script: &[u8],
    page: i64,
    page_size: i64,
    cota_id_opt: Option<[u8; 20]>,
) -> DBTotalResult<(WithdrawNFTDb, Option<ClassInfoDb>)> {
    let lock_hash = blake2b_256(lock_script);
    let script_id_opt = get_script_id_by_lock_script(lock_script)?;
    let (mut withdrawal_nfts, _, block_height) = match script_id_opt {
        Some(script_id) => get_withdrawal_cota_by_script_id(script_id, cota_id_opt),
        None => Ok((vec![], 0, 0)),
    }?;
    debug!("withdrawal_nfts: {:?}", withdrawal_nfts);
    withdrawal_nfts = withdrawal_nfts
        .into_iter()
        .filter(|withdrawal| {
            !is_exist_in_claim(
                lock_hash,
                withdrawal.cota_id,
                withdrawal.token_index,
                withdrawal.out_point,
            )
        })
        .collect();
    let total = withdrawal_nfts.len() as i64;

    let mut nfts: Vec<(WithdrawNFTDb, Option<ClassInfoDb>)> = vec![];
    let range = page * page_size..(page + 1) * page_size;
    for (index, withdrawal) in withdrawal_nfts.into_iter().enumerate() {
        if range.contains(&(index as i64)) {
            let class_info = get_class_info_by_cota_id(withdrawal.cota_id)?;
            nfts.push((withdrawal, class_info))
        }
    }
    debug!("withdraw: {:?}", nfts);

    Ok((nfts, total, block_height))
}

pub fn get_mint_cota(
    lock_script: &[u8],
    page: i64,
    page_size: i64,
) -> DBTotalResult<(WithdrawDb, Option<ClassInfoDb>)> {
    let lock_hash = blake2b_256(lock_script);
    let defines = get_define_cota_by_lock_hash(lock_hash)?.0;
    let cota_ids: Vec<[u8; 20]> = defines.into_iter().map(|define| define.cota_id).collect();
    let (withdrawal_nfts, total, block_height) =
        get_withdrawal_cota_by_cota_ids(lock_hash, cota_ids, page, page_size)?;
    let mut nfts: Vec<(WithdrawDb, Option<ClassInfoDb>)> = vec![];
    for withdrawal in withdrawal_nfts {
        let class_info = get_class_info_by_cota_id(withdrawal.cota_id)?;
        nfts.push((withdrawal, class_info))
    }
    Ok((nfts, total, block_height))
}

pub fn check_cota_claimed(
    lock_script: &[u8],
    cota_id: [u8; 20],
    index: [u8; 4],
) -> Result<(bool, u64), Error> {
    let lock_hash = blake2b_256(lock_script);
    check_hold_cota_by_lock_hash(lock_hash, (cota_id, index))
}

pub fn get_sender_lock_hash_by_cota_nft(
    lock_script: &[u8],
    cota_id: [u8; 20],
    token_index: [u8; 4],
) -> Result<Option<String>, Error> {
    let lock_script_id_opt = get_script_id_by_lock_script(lock_script)?;
    if lock_script_id_opt.is_none() {
        return Ok(None);
    }
    get_sender_lock_by_script_id(lock_script_id_opt.unwrap(), cota_id, token_index)
}

pub fn get_define_info_by_cota_id(
    cota_id: [u8; 20],
) -> Result<(Option<DefineDb>, Option<ClassInfoDb>), Error> {
    let define_opt: Option<DefineDb> = get_define_cota_by_cota_id(cota_id)?;
    let class_info_opt = get_class_info_by_cota_id(cota_id)?;
    Ok((define_opt, class_info_opt))
}

pub fn get_owned_cota_count(lock_script: &[u8], cota_id: [u8; 20]) -> Result<(i64, u64), Error> {
    let lock_hash = blake2b_256(lock_script);
    let script_id_opt = get_script_id_by_lock_script(lock_script)?;
    let (withdrawal_nfts, _, block_height) = match script_id_opt {
        Some(script_id) => get_withdrawal_cota_by_script_id(script_id, Some(cota_id)),
        None => Ok((vec![], 0, 0)),
    }?;
    let withdrawal_count = withdrawal_nfts
        .into_iter()
        .filter(|withdrawal| {
            !is_exist_in_claim(
                lock_hash,
                withdrawal.cota_id,
                withdrawal.token_index,
                withdrawal.out_point,
            )
        })
        .count() as i64;

    let hold_count = get_hold_cota_count_by_lock_hash(lock_hash, cota_id)?;
    debug!(
        "hold count: {} and withdrawal count: {}",
        hold_count, withdrawal_count
    );
    let count = hold_count + withdrawal_count;
    Ok((count, block_height))
}
