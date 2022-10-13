use crate::business::helper::address_from_script;
use crate::models::claim::{get_claim_cota_by_lock_hash, is_exist_in_claim, ClaimDb};
use crate::models::class::{get_class_info_by_cota_id, ClassInfoDb};
use crate::models::define::{get_define_cota_by_cota_id, get_define_cota_by_lock_hash, DefineDb};
use crate::models::hold::{
    check_hold_cota_by_lock_hash, get_hold_cota_by_lock_hash, get_hold_cota_by_lock_hash_and_page,
    get_hold_cota_count_by_lock_hash, HoldDb,
};
use crate::models::scripts::get_script_id_by_lock_script;
use crate::models::withdrawal::nft::{
    get_sender_lock_by_script_id, get_withdrawal_cota_by_cota_ids,
    get_withdrawal_cota_by_lock_hash, get_withdrawal_cota_by_script_id, WithdrawDb, WithdrawNFTDb,
};
use crate::models::DBTotalResult;
use crate::utils::error::Error;
use cota_smt::smt::blake2b_256;
use log::debug;

use super::define::get_lock_hash_by_cota_id;
use super::extension::leaves::{get_extension_leaves_by_lock_hash, ExtensionLeafDb};
use super::issuer::{get_issuer_info_by_lock_hash, IssuerInfoDb};
use super::joyid::{get_joyid_info_by_lock_hash, get_lock_hash_by_nickname};
use super::registry::get_ccid_lock_script;

type DBAllResult = Result<
    (
        Vec<DefineDb>,
        Vec<HoldDb>,
        Vec<WithdrawDb>,
        Vec<ClaimDb>,
        Vec<ExtensionLeafDb>,
    ),
    Error,
>;

pub fn get_all_cota_by_lock_hash(lock_hash: [u8; 32]) -> DBAllResult {
    let (defines, _) = get_define_cota_by_lock_hash(lock_hash)?;
    let (holds, _) = get_hold_cota_by_lock_hash(lock_hash, &vec![])?;
    let (withdrawals, _) = get_withdrawal_cota_by_lock_hash(lock_hash, &vec![])?;
    let (claims, _) = get_claim_cota_by_lock_hash(lock_hash)?;
    let (extension_leaves, _) = get_extension_leaves_by_lock_hash(lock_hash)?;
    Ok((defines, holds, withdrawals, claims, extension_leaves))
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
    check_hold_cota_by_lock_hash(blake2b_256(lock_script), (cota_id, index))
}

pub fn get_sender_account_by_cota_nft(
    lock_script: &[u8],
    cota_id: [u8; 20],
    token_index: [u8; 4],
) -> Result<Option<(String, Vec<u8>)>, Error> {
    match get_script_id_by_lock_script(lock_script)? {
        Some(script_id) => get_sender_lock_by_script_id(script_id, cota_id, token_index),
        None => Ok(None),
    }
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

pub fn get_issuer_by_cota_id(cota_id: [u8; 20]) -> Result<([u8; 32], Option<IssuerInfoDb>), Error> {
    let lock_hash = get_lock_hash_by_cota_id(cota_id)?;
    let issuer = get_issuer_info_by_lock_hash(lock_hash)?;
    Ok((lock_hash, issuer))
}

pub fn get_ccid_account(
    lock_hash_opt: Option<[u8; 32]>,
    ccid_opt: Option<u64>,
    nickname_opt: Option<String>,
) -> Result<Option<(String, u64, String)>, Error> {
    if lock_hash_opt.is_none() && ccid_opt.is_none() && nickname_opt.is_none() {
        return Err(Error::RequestParamTypeError(
            "lock hash, ccid and nickname cannot be all null".to_string(),
        ));
    }
    let parse_ccid_info = |lock_hash_opt: Option<[u8; 32]>, ccid_opt: Option<u64>| {
        let result = match get_ccid_lock_script(lock_hash_opt, ccid_opt)? {
            Some((lock_script, ccid)) => {
                let joyid_info = get_joyid_info_by_lock_hash(blake2b_256(&lock_script))?;
                let address = address_from_script(&lock_script)?;
                joyid_info.map(|info| (address, ccid, info.nickname))
            }
            None => None,
        };
        Ok(result)
    };
    if lock_hash_opt.is_some() || ccid_opt.is_some() {
        parse_ccid_info(lock_hash_opt, ccid_opt)
    } else {
        let lock_hash_ = get_lock_hash_by_nickname(&nickname_opt.unwrap())?;
        parse_ccid_info(lock_hash_, None)
    }
}
