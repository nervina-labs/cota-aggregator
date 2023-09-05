use crate::ckb::indexer::get_cota_smt_root;
use crate::entries::helper::with_lock;
use crate::entries::smt::{generate_history_smt, init_smt};
use crate::models::extension::social::get_social_config_by_lock;
use crate::models::extension::subkey::get_subkey_by_pubkey_hash;
use crate::request::social::{SocialFriend, SocialUnlockReq};
use crate::smt::transaction::store_transaction::StoreTransaction;
use crate::utils::error::Error;
use crate::utils::helper::blake2b_160;
use crate::ROCKS_DB;
use cota_smt::molecule::prelude::*;
use cota_smt::smt::blake2b_256;
use joyid_smt::common::*;
use joyid_smt::joyid::{
    FriendPubkeyBuilder, FriendPubkeyVec, FriendPubkeyVecBuilder, SocialUnlockEntries,
    SocialUnlockEntriesBuilder,
};
use log::error;

use super::helper::{
    generate_ext_social_key, generate_subkey_key, generate_unlock_social_value, vec_to_bytes,
};
use super::smt::generate_mysql_smt;

pub async fn generate_social_unlock_smt(
    social_unlock_req: SocialUnlockReq,
) -> Result<SocialUnlockEntries, Error> {
    let SocialUnlockReq {
        friends,
        lock_script,
    } = social_unlock_req;
    let lock_hash = blake2b_256(lock_script.clone());

    let social = get_social_config_by_lock(lock_hash)?.ok_or(Error::SocialLeafNotFound)?;
    let (_, key) = generate_ext_social_key();
    let (social_value, _) = generate_unlock_social_value(&social);

    let smt_root = get_cota_smt_root(&lock_script).await?;
    let transaction = &StoreTransaction::new(ROCKS_DB.transaction());

    let mut smt = init_smt(transaction, lock_hash)?;
    // Add lock to smt
    with_lock(lock_hash, || {
        generate_history_smt(&mut smt, lock_hash, smt_root)
    })?;

    let social_merkle_proof = smt.merkle_proof(vec![key]).map_err(|e| {
        error!("Social unlock SMT proof error: {:?}", e.to_string());
        Error::SMTProofInvalid("Social unlock".to_string())
    })?;
    let social_merkle_proof_compiled = social_merkle_proof.compile(vec![key]).map_err(|e| {
        error!("Social unlock SMT proof error: {:?}", e.to_string());
        Error::SMTProofInvalid("Social unlock".to_string())
    })?;

    let merkel_proof_vec: Vec<u8> = social_merkle_proof_compiled.into();
    let merkel_proof_bytes = BytesBuilder::default()
        .extend(merkel_proof_vec.iter().map(|v| Byte::from(*v)))
        .build();

    let unlock_entries = SocialUnlockEntriesBuilder::default()
        .social_value(social_value)
        .social_proof(merkel_proof_bytes)
        .social_friends(generate_social_friends(friends)?)
        .build();

    Ok(unlock_entries)
}

fn generate_social_friends(friends: Vec<SocialFriend>) -> Result<FriendPubkeyVec, Error> {
    let mut friend_pubkeys = Vec::with_capacity(friends.len());
    for friend in friends {
        if friend.unlock_mode != 1 && friend.unlock_mode != 2 {
            return Err(Error::SocialFriendInfoInvalid("Unlock mode".to_owned()));
        }
        if friend.unlock_mode == 1 {
            let mut friend_pubkey = FriendPubkeyBuilder::default()
                .unlock_mode(Byte::from_slice(&[friend.unlock_mode]).unwrap())
                .alg_index(Uint16::from_slice(&friend.alg_index.to_be_bytes()).unwrap())
                .pubkey(vec_to_bytes(&friend.pubkey))
                .signature(vec_to_bytes(&friend.signature))
                .build();
            if friend.alg_index == 1 || friend.alg_index == 3 {
                if friend.web_authn_msg.is_empty() {
                    return Err(Error::RequestParamNotFound("web_authn_msg".to_string()));
                }
                friend_pubkey = friend_pubkey
                    .as_builder()
                    .web_authn_msg(vec_to_bytes(&friend.web_authn_msg))
                    .build();
            }
            friend_pubkeys.push(friend_pubkey);
        } else {
            let lock_hash = blake2b_256(&friend.lock_script);
            let pubkey_hash = if friend.alg_index == 2 {
                let mut hash = [0u8; 20];
                hash.copy_from_slice(&friend.pubkey);
                hash
            } else {
                blake2b_160(&friend.pubkey)
            };

            let subkey = get_subkey_by_pubkey_hash(lock_hash, pubkey_hash, friend.alg_index)?
                .ok_or(Error::SubkeyLeafNotFound)?;
            let (_, key) = generate_subkey_key(subkey.ext_data);
            let ext_data = Uint32::from_slice(&subkey.ext_data.to_be_bytes())
                .map_err(|_| Error::Other("Parse uint32 error".to_owned()))?;
            let alg_index = Uint16::from_slice(&subkey.alg_index.to_be_bytes())
                .map_err(|_| Error::Other("Parse uint16 error".to_owned()))?;

            let transaction = &StoreTransaction::new(ROCKS_DB.transaction());

            let mut smt = init_smt(transaction, lock_hash)?;
            // Add lock to smt
            with_lock(lock_hash, || generate_mysql_smt(&mut smt, lock_hash))?;

            let subkey_merkle_proof = smt.merkle_proof(vec![key]).map_err(|e| {
                error!("Friend subkey SMT proof error: {:?}", e.to_string());
                Error::SMTProofInvalid("Friend subkey".to_string())
            })?;
            let subkey_merkle_proof_compiled =
                subkey_merkle_proof.compile(vec![key]).map_err(|e| {
                    error!("Friend subkey SMT proof error: {:?}", e.to_string());
                    Error::SMTProofInvalid("Friend subkey".to_string())
                })?;

            let merkel_proof_vec: Vec<u8> = subkey_merkle_proof_compiled.into();
            let merkel_proof_bytes = BytesBuilder::default()
                .extend(merkel_proof_vec.iter().map(|v| Byte::from(*v)))
                .build();

            let mut friend_pubkey = FriendPubkeyBuilder::default()
                .unlock_mode(Byte::from_slice(&[friend.unlock_mode]).unwrap())
                .alg_index(alg_index)
                .pubkey(vec_to_bytes(&friend.pubkey))
                .signature(vec_to_bytes(&friend.signature))
                .ext_data(ext_data)
                .subkey_proof(merkel_proof_bytes)
                .build();
            if friend.alg_index == 1 || friend.alg_index == 3 {
                if friend.web_authn_msg.is_empty() {
                    return Err(Error::RequestParamNotFound("web_authn_msg".to_string()));
                }
                friend_pubkey = friend_pubkey
                    .as_builder()
                    .web_authn_msg(vec_to_bytes(&friend.web_authn_msg))
                    .build();
            }
            friend_pubkeys.push(friend_pubkey);
        }
    }
    let friend_pubkey_vec = FriendPubkeyVecBuilder::default()
        .set(friend_pubkeys)
        .build();
    Ok(friend_pubkey_vec)
}
