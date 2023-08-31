use super::constants::EXT_ACTION_ADD;
use super::helper::{
    generate_ext_social_key, generate_ext_social_value, generate_subkey_key, generate_subkey_value,
};
use crate::ckb::indexer::get_cota_smt_root;
use crate::entries::helper::with_lock;
use crate::entries::smt::{generate_history_smt, init_smt};
use crate::models::extension::leaves::get_extension_leaf_by_lock_hash;
use crate::request::extension::{ExtSocialReq, ExtSubkeysReq};
use crate::smt::transaction::store_transaction::StoreTransaction;
use crate::smt::RootSaver;
use crate::utils::error::Error;
use crate::ROCKS_DB;
use cota_smt::common::*;
use cota_smt::extension::{
    ExtensionEntries, ExtensionEntriesBuilder, ExtensionLeavesBuilder, ExtensionVecBuilder,
};
use cota_smt::molecule::prelude::*;
use cota_smt::smt::{blake2b_256, H256};
use joyid_smt::joyid::{
    SocialEntryBuilder, SubKeyEntriesBuilder, SubKeyVecBuilder, SubValueVecBuilder,
};
use log::error;

pub async fn generate_ext_subkey_smt(
    ext_subkey_req: ExtSubkeysReq,
) -> Result<(H256, ExtensionEntries), Error> {
    let ExtSubkeysReq {
        lock_script,
        subkeys,
        ext_action,
    } = ext_subkey_req;
    let subkey_len = subkeys.len();
    let mut update_leaves: Vec<(H256, H256)> = Vec::with_capacity(subkey_len);
    let mut previous_leaves: Vec<(H256, H256)> = Vec::with_capacity(subkey_len);

    let lock_hash = blake2b_256(&lock_script);

    let mut sub_keys = vec![];
    let mut sub_values = vec![];
    for index in 0..subkey_len {
        let subkey = subkeys.get(index).unwrap();
        let (sub_key, key) = generate_subkey_key(subkey.ext_data);
        let (sub_value, value) = generate_subkey_value(subkey);
        sub_keys.push(sub_key);
        sub_values.push(sub_value);
        update_leaves.push((key, value));
        if ext_action == EXT_ACTION_ADD {
            previous_leaves.push((key, H256::zero()));
        } else {
            let leaf_opt = get_extension_leaf_by_lock_hash(lock_hash, key)?;
            if let Some(leaf) = leaf_opt {
                previous_leaves.push((key, H256::from(leaf.value)));
            } else {
                return Err(Error::CKBRPCError(
                    "Extension old subkey value does not exist".to_string(),
                ));
            }
        }
    }

    let smt_root = get_cota_smt_root(&lock_script).await?;
    let transaction = &StoreTransaction::new(ROCKS_DB.transaction());

    let mut smt = init_smt(transaction, lock_hash)?;
    // Add lock to smt
    with_lock(lock_hash, || {
        generate_history_smt(&mut smt, lock_hash, smt_root)?;
        smt.update_all(update_leaves.clone())
            .map_err(|e| Error::SMTError(e.to_string()))?;
        smt.save_root_and_leaves(previous_leaves.clone())?;
        smt.commit()
    })?;

    let leaf_keys: Vec<H256> = update_leaves.iter().map(|leave| leave.0).collect();
    let extension_merkle_proof = smt.merkle_proof(leaf_keys.clone()).map_err(|e| {
        error!("Extension subkey SMT proof error: {:?}", e.to_string());
        Error::SMTProofError("Extension subkey".to_string())
    })?;
    let extension_merkle_proof_compiled =
        extension_merkle_proof.compile(leaf_keys).map_err(|e| {
            error!("Extension subkey SMT proof error: {:?}", e.to_string());
            Error::SMTProofError("Extension subkey".to_string())
        })?;

    let merkel_proof_vec: Vec<u8> = extension_merkle_proof_compiled.into();
    let merkel_proof_bytes = BytesBuilder::default()
        .extend(merkel_proof_vec.iter().map(|v| Byte::from(*v)))
        .build();

    let subkey_entries = SubKeyEntriesBuilder::default()
        .keys(SubKeyVecBuilder::default().set(sub_keys.clone()).build())
        .values(
            SubValueVecBuilder::default()
                .set(sub_values.clone())
                .build(),
        )
        .build();
    let ext_sub_keys = sub_keys
        .into_iter()
        .map(|key| Byte32::from_slice(key.as_slice()).unwrap())
        .collect();
    let ext_sub_values = sub_values
        .into_iter()
        .map(|value| Byte32::from_slice(value.as_slice()).unwrap())
        .collect();
    let prev_sub_values = previous_leaves
        .into_iter()
        .map(|leaf| Byte32::from_slice(leaf.1.as_slice()).unwrap())
        .collect();
    let ext_leaves = ExtensionLeavesBuilder::default()
        .keys(ExtensionVecBuilder::default().set(ext_sub_keys).build())
        .values(ExtensionVecBuilder::default().set(ext_sub_values).build())
        .old_values(ExtensionVecBuilder::default().set(prev_sub_values).build())
        .proof(merkel_proof_bytes)
        .build();

    let ext_raw_data = BytesBuilder::default()
        .extend(subkey_entries.as_slice().iter().map(|v| Byte::from(*v)))
        .build();
    let extension_entries = ExtensionEntriesBuilder::default()
        .leaves(ext_leaves)
        .sub_type(Byte6::from_slice("subkey".as_bytes()).unwrap())
        .raw_data(ext_raw_data)
        .build();

    Ok((*smt.root(), extension_entries))
}

pub async fn generate_ext_social_smt(
    ext_social_req: ExtSocialReq,
) -> Result<(H256, ExtensionEntries), Error> {
    let mut update_leaves: Vec<(H256, H256)> = Vec::with_capacity(1);
    let mut previous_leaves: Vec<(H256, H256)> = Vec::with_capacity(1);

    let lock_hash = blake2b_256(&ext_social_req.lock_script);

    let (social_key, key) = generate_ext_social_key();
    let (social_value, value) = generate_ext_social_value(&ext_social_req);
    update_leaves.push((key, value));

    if ext_social_req.ext_action == EXT_ACTION_ADD {
        previous_leaves.push((key, H256::zero()));
    } else {
        let leaf_opt = get_extension_leaf_by_lock_hash(lock_hash, key)?;
        if let Some(leaf) = leaf_opt {
            previous_leaves.push((key, H256::from(leaf.value)));
        } else {
            return Err(Error::CKBRPCError(
                "Extension old social value does not exist".to_string(),
            ));
        }
    }

    let smt_root = get_cota_smt_root(&ext_social_req.lock_script).await?;
    let transaction = &StoreTransaction::new(ROCKS_DB.transaction());

    let mut smt = init_smt(transaction, lock_hash)?;
    // Add lock to smt
    with_lock(lock_hash, || {
        generate_history_smt(&mut smt, lock_hash, smt_root)?;
        smt.update_all(update_leaves.clone())
            .map_err(|e| Error::SMTError(e.to_string()))?;
        smt.save_root_and_leaves(previous_leaves.clone())?;
        smt.commit()
    })?;

    let leaf_keys: Vec<H256> = update_leaves.iter().map(|leave| leave.0).collect();
    let extension_merkle_proof = smt.merkle_proof(leaf_keys.clone()).map_err(|e| {
        error!("Extension social SMT proof error: {:?}", e.to_string());
        Error::SMTProofError("Extension social".to_string())
    })?;
    let extension_merkle_proof_compiled =
        extension_merkle_proof.compile(leaf_keys).map_err(|e| {
            error!("Extension social SMT proof error: {:?}", e.to_string());
            Error::SMTProofError("Extension social".to_string())
        })?;

    let merkel_proof_vec: Vec<u8> = extension_merkle_proof_compiled.into();
    let merkel_proof_bytes = BytesBuilder::default()
        .extend(merkel_proof_vec.iter().map(|v| Byte::from(*v)))
        .build();

    let ext_social_keys = vec![Byte32::from_slice(social_key.as_slice()).unwrap()];
    let ext_social_values = vec![Byte32::from_slice(value.as_slice()).unwrap()];
    let social_entry = SocialEntryBuilder::default()
        .key(social_key)
        .value(social_value)
        .build();
    let prev_social_values = previous_leaves
        .into_iter()
        .map(|leaf| Byte32::from_slice(leaf.1.as_slice()).unwrap())
        .collect();
    let ext_leaves = ExtensionLeavesBuilder::default()
        .keys(ExtensionVecBuilder::default().set(ext_social_keys).build())
        .values(
            ExtensionVecBuilder::default()
                .set(ext_social_values)
                .build(),
        )
        .old_values(
            ExtensionVecBuilder::default()
                .set(prev_social_values)
                .build(),
        )
        .proof(merkel_proof_bytes)
        .build();

    let ext_raw_data = BytesBuilder::default()
        .extend(social_entry.as_slice().iter().map(|v| Byte::from(*v)))
        .build();
    let extension_entries = ExtensionEntriesBuilder::default()
        .leaves(ext_leaves)
        .sub_type(Byte6::from_slice("social".as_bytes()).unwrap())
        .raw_data(ext_raw_data)
        .build();

    Ok((*smt.root(), extension_entries))
}

pub async fn generate_adding_subkey_smt(
    ext_subkey_req: ExtSubkeysReq,
) -> Result<(H256, ExtensionEntries), Error> {
    let ExtSubkeysReq {
        lock_script,
        subkeys,
        ..
    } = ext_subkey_req;
    let subkey_len = subkeys.len();
    let mut update_leaves: Vec<(H256, H256)> = Vec::with_capacity(subkey_len);

    let lock_hash = blake2b_256(&lock_script);

    let mut sub_keys = vec![];
    let mut sub_values = vec![];
    for index in 0..subkey_len {
        let subkey = subkeys.get(index).unwrap();
        let (sub_key, key) = generate_subkey_key(subkey.ext_data);
        let (sub_value, value) = generate_subkey_value(subkey);
        sub_keys.push(sub_key);
        sub_values.push(sub_value);
        update_leaves.push((key, value));
    }

    let transaction = &StoreTransaction::new(ROCKS_DB.transaction());
    let mut smt = init_smt(transaction, lock_hash)?;
    smt.update_all(update_leaves.clone())
        .map_err(|e| Error::SMTError(e.to_string()))?;

    let leaf_keys: Vec<H256> = update_leaves.iter().map(|leave| leave.0).collect();
    let extension_merkle_proof = smt.merkle_proof(leaf_keys.clone()).map_err(|e| {
        error!("Extension subkey SMT proof error: {:?}", e.to_string());
        Error::SMTProofError("Extension subkey".to_string())
    })?;
    let extension_merkle_proof_compiled =
        extension_merkle_proof.compile(leaf_keys).map_err(|e| {
            error!("Extension subkey SMT proof error: {:?}", e.to_string());
            Error::SMTProofError("Extension subkey".to_string())
        })?;

    let merkel_proof_vec: Vec<u8> = extension_merkle_proof_compiled.into();
    let merkel_proof_bytes = BytesBuilder::default()
        .extend(merkel_proof_vec.iter().map(|v| Byte::from(*v)))
        .build();

    let subkey_entries = SubKeyEntriesBuilder::default()
        .keys(SubKeyVecBuilder::default().set(sub_keys.clone()).build())
        .values(
            SubValueVecBuilder::default()
                .set(sub_values.clone())
                .build(),
        )
        .build();
    let ext_sub_keys = sub_keys
        .into_iter()
        .map(|key| Byte32::from_slice(key.as_slice()).unwrap())
        .collect();
    let ext_sub_values = sub_values
        .into_iter()
        .map(|value| Byte32::from_slice(value.as_slice()).unwrap())
        .collect();
    let ext_leaves = ExtensionLeavesBuilder::default()
        .keys(ExtensionVecBuilder::default().set(ext_sub_keys).build())
        .values(ExtensionVecBuilder::default().set(ext_sub_values).build())
        .old_values(ExtensionVecBuilder::default().build())
        .proof(merkel_proof_bytes)
        .build();

    let ext_raw_data = BytesBuilder::default()
        .extend(subkey_entries.as_slice().iter().map(|v| Byte::from(*v)))
        .build();
    let extension_entries = ExtensionEntriesBuilder::default()
        .leaves(ext_leaves)
        .sub_type(Byte6::from_slice("subkey".as_bytes()).unwrap())
        .raw_data(ext_raw_data)
        .build();

    Ok((*smt.root(), extension_entries))
}
