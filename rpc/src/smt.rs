use crate::db::get_registry_lock_hashes;
use cota_smt::common::{Byte32, BytesBuilder};
use cota_smt::molecule::prelude::*;
use cota_smt::registry::{
    CotaNFTRegistryEntriesBuilder, Registry, RegistryBuilder, RegistryVecBuilder,
};
use cota_smt::smt::{Blake2bHasher, H256, SMT};

pub fn generate_registry_smt(lock_hashes: Vec<[u8; 32]>) -> (String, String) {
    let mut smt = SMT::default();
    let update_leaves_count = lock_hashes.len();

    let registered_lock_hashes = get_registry_lock_hashes();
    if !registered_lock_hashes.is_empty() {
        for lock_hash in registered_lock_hashes {
            let key: H256 = H256::from(lock_hash);
            let value: H256 = H256::from([255u8; 32]);
            smt.update(key, value).expect("SMT update leave error");
        }
    }

    let mut update_leaves: Vec<(H256, H256)> = Vec::with_capacity(update_leaves_count);
    for lock_hash in lock_hashes {
        let key: H256 = H256::from(lock_hash);
        let value: H256 = H256::from([255u8; 32]);
        update_leaves.push((key, value));
        smt.update(key, value).expect("SMT update leave error");
    }
    let root_hash = smt.root().clone();

    let mut root_hash_bytes = [0u8; 32];
    root_hash_bytes.copy_from_slice(root_hash.as_slice());
    let root_hash_hex = hex::encode(root_hash_bytes);

    println!("smt root hash: {:?}", root_hash_hex);

    let registry_merkle_proof = smt
        .merkle_proof(update_leaves.iter().map(|leave| leave.0).collect())
        .unwrap();
    let registry_merkle_proof_compiled = registry_merkle_proof
        .compile(update_leaves.clone())
        .unwrap();
    let verify_result = registry_merkle_proof_compiled
        .verify::<Blake2bHasher>(&root_hash, update_leaves.clone())
        .expect("smt proof verify failed");
    assert!(verify_result, "smt proof verify failed");

    let merkel_proof_vec: Vec<u8> = registry_merkle_proof_compiled.into();

    println!("smt proof: {:?}", merkel_proof_vec);

    let registry_vec = update_leaves
        .iter()
        .map(|leave| {
            let key: [u8; 32] = leave.0.into();
            let value: [u8; 32] = leave.1.into();
            RegistryBuilder::default()
                .lock_hash(Byte32::from_slice(&key).unwrap())
                .state(Byte32::from_slice(&value).unwrap())
                .build()
        })
        .collect::<Vec<Registry>>();

    let registry_vec = RegistryVecBuilder::default().extend(registry_vec).build();
    let merkel_proof_bytes = BytesBuilder::default()
        .extend(merkel_proof_vec.iter().map(|v| Byte::from(*v)))
        .build();

    let registry_entries = CotaNFTRegistryEntriesBuilder::default()
        .registries(registry_vec)
        .proof(merkel_proof_bytes)
        .build();

    let registry_entries_hex = hex::encode(registry_entries.as_slice());

    println!("registry_entries_hex: {:?}", registry_entries_hex);

    (root_hash_hex, registry_entries_hex)
}
