use crate::models::claim::{get_claim_cota_by_lock_hash, ClaimDb};
use crate::models::define::{get_define_cota_by_lock_hash, DefineDb};
use crate::models::hold::{get_hold_cota_by_lock_hash, HoldDb};
use crate::models::withdrawal::{get_withdrawal_cota_by_lock_hash, WithdrawDb};
use crate::utils::error::Error;

pub fn get_all_cota_by_lock_hash(
    lock_hash: [u8; 32],
) -> Result<(Vec<DefineDb>, Vec<HoldDb>, Vec<WithdrawDb>, Vec<ClaimDb>), Error> {
    let defines = get_define_cota_by_lock_hash(lock_hash)?;
    let holds = get_hold_cota_by_lock_hash(lock_hash, None)?;
    let withdrawals = get_withdrawal_cota_by_lock_hash(lock_hash, None)?;
    let claims = get_claim_cota_by_lock_hash(lock_hash)?;
    Ok((defines, holds, withdrawals, claims))
}
