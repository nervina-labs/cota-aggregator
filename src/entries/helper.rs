use crate::entries::constants::{
    BLOCK_HEIGHT_VALUE_PADDING_MAINNET, BLOCK_HEIGHT_VALUE_PADDING_TESTNET, CLAIM_NFT_SMT_TYPE,
    DEFINE_NFT_SMT_TYPE, HOLD_NFT_SMT_TYPE, WITHDRAWAL_NFT_SMT_TYPE,
};
use crate::entries::SMT_LOCK;
use crate::models::extension::social::SocialRecoveryDb;
use crate::request::extension::{ExtSocialReq, ExtSubkey};
use crate::utils::error::Error;
use cota_smt::common::{Uint16, Uint32, *};
use cota_smt::molecule::prelude::*;
use cota_smt::smt::{blake2b_256, H256};
use joyid_smt::common;
use joyid_smt::joyid::{
    LockScriptVecBuilder, SocialKey, SocialValue, SocialValueBuilder, SubKey, SubValue,
};
use serde_json::from_str;
use std::env;
use std::sync::Arc;

pub fn generate_define_key(cota_id: [u8; 20]) -> (DefineCotaNFTId, H256) {
    let cota_id = CotaId::from_slice(&cota_id).unwrap();
    let smt_type = Uint16::from_slice(&DEFINE_NFT_SMT_TYPE).unwrap();
    let define_key = DefineCotaNFTIdBuilder::default()
        .cota_id(cota_id)
        .smt_type(smt_type)
        .build();
    let mut define_key_bytes = [0u8; 32];
    define_key_bytes[0..22].copy_from_slice(define_key.as_slice());
    let key = H256::from(define_key_bytes);
    (define_key, key)
}

pub fn generate_define_value(
    total: [u8; 4],
    issued: [u8; 4],
    configure: u8,
    block_number: u64,
) -> (DefineCotaNFTValue, H256) {
    let after_padding = block_number > get_value_padding_block_height();
    let define_value = DefineCotaNFTValueBuilder::default()
        .total(Uint32::from_slice(&total).unwrap())
        .issued(Uint32::from_slice(&issued).unwrap())
        .configure(Byte::from(configure))
        .build();
    let mut define_value_bytes = [0u8; 32];
    define_value_bytes[0..9].copy_from_slice(define_value.as_slice());
    if after_padding || define_value_bytes == [0u8; 32] {
        define_value_bytes[31] = 255u8;
    }

    let value = H256::from(define_value_bytes);
    (define_value, value)
}

pub fn generate_hold_key(cota_id: [u8; 20], token_index: [u8; 4]) -> (CotaNFTId, H256) {
    let hold_key = CotaNFTIdBuilder::default()
        .cota_id(CotaId::from_slice(&cota_id).unwrap())
        .smt_type(Uint16::from_slice(&HOLD_NFT_SMT_TYPE).unwrap())
        .index(Uint32::from_slice(&token_index).unwrap())
        .build();
    let mut hold_key_bytes = [0u8; 32];
    hold_key_bytes[0..26].copy_from_slice(hold_key.as_slice());
    let key = H256::from(hold_key_bytes);
    (hold_key, key)
}

pub fn generate_hold_value(
    configure: u8,
    state: u8,
    characteristic: [u8; 20],
) -> (CotaNFTInfo, H256) {
    let hold_value = CotaNFTInfoBuilder::default()
        .characteristic(Characteristic::from_slice(&characteristic).unwrap())
        .configure(Byte::from(configure))
        .state(Byte::from(state))
        .build();
    let mut hold_value_bytes = [0u8; 32];
    hold_value_bytes[0..22].copy_from_slice(hold_value.as_slice());
    hold_value_bytes[31] = 255u8;
    let value = H256::from(hold_value_bytes);
    (hold_value, value)
}

pub fn generate_withdrawal_key(cota_id: [u8; 20], token_index: [u8; 4]) -> (CotaNFTId, H256) {
    let withdrawal_key = CotaNFTIdBuilder::default()
        .cota_id(CotaId::from_slice(&cota_id).unwrap())
        .smt_type(Uint16::from_slice(&WITHDRAWAL_NFT_SMT_TYPE).unwrap())
        .index(Uint32::from_slice(&token_index).unwrap())
        .build();
    let mut withdrawal_key_bytes = [0u8; 32];
    withdrawal_key_bytes[0..26].copy_from_slice(withdrawal_key.as_slice());
    let key = H256::from(withdrawal_key_bytes);

    (withdrawal_key, key)
}

pub fn generate_withdrawal_key_v1(
    cota_id: [u8; 20],
    token_index: [u8; 4],
    out_point: [u8; 24],
) -> (WithdrawalCotaNFTKeyV1, H256) {
    let nft_id = CotaNFTIdBuilder::default()
        .cota_id(CotaId::from_slice(&cota_id).unwrap())
        .smt_type(Uint16::from_slice(&WITHDRAWAL_NFT_SMT_TYPE).unwrap())
        .index(Uint32::from_slice(&token_index).unwrap())
        .build();
    let withdrawal_key = WithdrawalCotaNFTKeyV1Builder::default()
        .nft_id(nft_id)
        .out_point(OutPointSlice::from_slice(&out_point).unwrap())
        .build();
    let key = H256::from(blake2b_256(withdrawal_key.as_slice()));

    (withdrawal_key, key)
}

pub fn generate_withdrawal_value(
    configure: u8,
    state: u8,
    characteristic: [u8; 20],
    to_lock_script: &[u8],
    out_point: [u8; 24],
) -> (WithdrawalCotaNFTValue, H256) {
    let cota_info = CotaNFTInfoBuilder::default()
        .configure(Byte::from(configure))
        .state(Byte::from(state))
        .characteristic(Characteristic::from_slice(&characteristic).unwrap())
        .build();
    let to_lock_bytes: Vec<Byte> = to_lock_script.iter().map(|v| Byte::from(*v)).collect();
    let withdrawal_value = WithdrawalCotaNFTValueBuilder::default()
        .nft_info(cota_info)
        .out_point(OutPointSlice::from_slice(&out_point).unwrap())
        .to_lock(BytesBuilder::default().set(to_lock_bytes).build())
        .build();
    let value = H256::from(blake2b_256(withdrawal_value.as_slice()));
    (withdrawal_value, value)
}

pub fn generate_withdrawal_value_v1(
    configure: u8,
    state: u8,
    characteristic: [u8; 20],
    to_lock_script: &[u8],
) -> (WithdrawalCotaNFTValueV1, H256) {
    let cota_info = CotaNFTInfoBuilder::default()
        .configure(Byte::from(configure))
        .state(Byte::from(state))
        .characteristic(Characteristic::from_slice(&characteristic).unwrap())
        .build();
    let to_lock_bytes: Vec<Byte> = to_lock_script.iter().map(|v| Byte::from(*v)).collect();
    let withdrawal_value = WithdrawalCotaNFTValueV1Builder::default()
        .nft_info(cota_info)
        .to_lock(BytesBuilder::default().set(to_lock_bytes).build())
        .build();
    let value = H256::from(blake2b_256(withdrawal_value.as_slice()));
    (withdrawal_value, value)
}

pub fn generate_claim_key(
    cota_id: [u8; 20],
    token_index: [u8; 4],
    out_point: [u8; 24],
) -> (ClaimCotaNFTKey, H256) {
    let nft_id = CotaNFTIdBuilder::default()
        .smt_type(Uint16::from_slice(&CLAIM_NFT_SMT_TYPE).unwrap())
        .cota_id(CotaId::from_slice(&cota_id).unwrap())
        .index(Uint32::from_slice(&token_index).unwrap())
        .build();
    let claimed_key = ClaimCotaNFTKeyBuilder::default()
        .nft_id(nft_id)
        .out_point(OutPointSlice::from_slice(&out_point).unwrap())
        .build();
    let key = H256::from(blake2b_256(claimed_key.as_slice()));
    (claimed_key, key)
}

pub fn generate_claim_value(version: u8) -> (Byte32, H256) {
    let mut claim_value_vec = vec![255u8; 31];
    if version == 0 {
        claim_value_vec.insert(0, 0u8);
    } else {
        claim_value_vec.insert(0, 1u8);
    }
    let claim_value = Byte32::from_slice(&claim_value_vec).unwrap();
    let value = H256::from([255u8; 32]);
    (claim_value, value)
}

pub fn generate_empty_value() -> (Byte32, H256) {
    let empty_value = Byte32Builder::default().set([Byte::from(0u8); 32]).build();
    let value = H256::zero();
    (empty_value, value)
}

pub fn generate_subkey_key(ext_data: u32) -> (SubKey, H256) {
    let mut ext_key = [0x00u8; 32];
    ext_key[0] = 0xFF;
    ext_key[2..8].copy_from_slice("subkey".as_bytes());
    ext_key[8..12].copy_from_slice(&ext_data.to_be_bytes());
    let sub_key = SubKey::from_slice(&ext_key).unwrap();
    (sub_key, H256::from(ext_key))
}

pub fn generate_subkey_value(subkey: &ExtSubkey) -> (SubValue, H256) {
    let mut ext_value = [0x00u8; 32];
    ext_value[0..2].copy_from_slice(&subkey.alg_index.to_be_bytes());
    ext_value[2..22].copy_from_slice(&subkey.pubkey_hash);
    ext_value[31] = 0xFF;
    let sub_value = SubValue::from_slice(&ext_value).unwrap();
    (sub_value, H256::from(ext_value))
}

pub fn generate_ext_social_key() -> (SocialKey, H256) {
    let mut ext_key = [0x00u8; 32];
    ext_key[0] = 0xFF;
    ext_key[2..8].copy_from_slice("social".as_bytes());
    let social_key = SocialKey::from_slice(&ext_key).unwrap();
    (social_key, H256::from(ext_key))
}

pub fn generate_ext_social_value(social: &ExtSocialReq) -> (SocialValue, H256) {
    let ExtSocialReq {
        recovery_mode,
        must,
        total,
        signers,
        ..
    } = social;
    generate_social_leaf_value(recovery_mode, must, total, signers)
}

pub fn generate_unlock_social_value(social: &SocialRecoveryDb) -> (SocialValue, H256) {
    let SocialRecoveryDb {
        recovery_mode,
        must,
        total,
        signers,
        ..
    } = social;
    generate_social_leaf_value(recovery_mode, must, total, signers)
}

fn generate_social_leaf_value(
    recovery_mode: &u8,
    must: &u8,
    total: &u8,
    signers: &[Vec<u8>],
) -> (SocialValue, H256) {
    let friends: Vec<common::Bytes> = signers.iter().map(|signer| vec_to_bytes(signer)).collect();
    let signers = LockScriptVecBuilder::default().set(friends).build();
    let social_value = SocialValueBuilder::default()
        .recovery_mode(Byte::from_slice(&[*recovery_mode]).unwrap())
        .must(Byte::from_slice(&[*must]).unwrap())
        .total(Byte::from_slice(&[*total]).unwrap())
        .signers(signers)
        .build();
    let value = H256::from(blake2b_256(social_value.as_slice()));
    (social_value, value)
}

pub fn generate_cota_index(cota_id: [u8; 20], token_index: [u8; 4]) -> Vec<u8> {
    let mut cota_id_index = vec![];
    cota_id_index.extend(&cota_id);
    cota_id_index.extend(&token_index);
    cota_id_index
}

pub fn with_lock<F>(lock_hash: [u8; 32], mut operator: F) -> Result<(), Error>
where
    F: FnMut() -> Result<(), Error>,
{
    let (lock, cond) = &*Arc::clone(&SMT_LOCK);
    {
        let mut set = lock.lock();
        while !set.insert(lock_hash) {
            cond.wait(&mut set);
        }
    }
    let unlock = || {
        let mut set = lock.lock();
        set.remove(&lock_hash);
        cond.notify_all();
    };
    let err_handle = |err| {
        unlock();
        err
    };
    operator().map_err(err_handle)?;
    unlock();
    Ok(())
}

pub fn get_value_padding_block_height() -> u64 {
    let is_mainnet: bool = match env::var("IS_MAINNET") {
        Ok(mainnet) => from_str::<bool>(&mainnet).unwrap(),
        Err(_e) => false,
    };
    if is_mainnet {
        BLOCK_HEIGHT_VALUE_PADDING_MAINNET
    } else {
        BLOCK_HEIGHT_VALUE_PADDING_TESTNET
    }
}

pub fn vec_to_bytes(values: &[u8]) -> joyid_smt::common::Bytes {
    joyid_smt::common::BytesBuilder::default()
        .set(values.iter().map(|v| Byte::from(*v)).collect())
        .build()
}
