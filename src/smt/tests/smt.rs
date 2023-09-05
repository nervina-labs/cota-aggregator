extern crate test;

use crate::smt::db::schema::{
    COLUMN_SMT_BRANCH, COLUMN_SMT_LEAF, COLUMN_SMT_ROOT, COLUMN_SMT_TEMP_LEAVES,
};
use crate::smt::store::smt_store::SMTStore;
use crate::smt::transaction::store_transaction::StoreTransaction;
use crate::smt::CotaSMT;
use crate::RocksDB;
use chrono::prelude::*;
use cota_smt::smt::{H256, SMT};
use rand::{thread_rng, Rng};

fn generate_smt(history_leaf_count: u32, update_leaf_count: u32) {
    let mut rng = thread_rng();
    let lock_hash: [u8; 32] = rng.gen::<[u8; 32]>();
    let mut leaves = vec![];
    for _ in 0..history_leaf_count {
        let key: H256 = rng.gen::<[u8; 32]>().into();
        let value: H256 = rng.gen::<[u8; 32]>().into();
        leaves.push((key, value));
    }
    let db = RocksDB::new_with_path(&format!("test_db_{}", history_leaf_count))
        .expect("rocksdb open error");
    let transaction = StoreTransaction::new(db.transaction());
    let smt_store = SMTStore::new(
        lock_hash,
        COLUMN_SMT_LEAF,
        COLUMN_SMT_BRANCH,
        COLUMN_SMT_ROOT,
        COLUMN_SMT_TEMP_LEAVES,
        &transaction,
    );
    let mut history_smt = CotaSMT::new(H256::zero(), smt_store);
    history_smt
        .update_all(leaves.clone())
        .expect("smt update leaves error");

    let mut update_leaves = vec![];
    for _ in 0..update_leaf_count {
        let key: H256 = rng.gen::<[u8; 32]>().into();
        let value: H256 = rng.gen::<[u8; 32]>().into();
        update_leaves.push((key, value));
    }

    let start_time = Local::now().timestamp_millis();
    let mut default_smt = SMT::default();
    default_smt
        .update_all(leaves.clone())
        .expect("smt update leaves error");
    default_smt
        .update_all(update_leaves.clone())
        .expect("smt update leaves error");
    default_smt
        .merkle_proof(update_leaves.iter().map(|leave| leave.0).collect())
        .expect("smt proof error");
    let default_diff_time = (Local::now().timestamp_millis() - start_time) as f64 / 1000f64;

    let start_time = Local::now().timestamp_millis();
    let smt_store = SMTStore::new(
        lock_hash,
        COLUMN_SMT_LEAF,
        COLUMN_SMT_BRANCH,
        COLUMN_SMT_ROOT,
        COLUMN_SMT_TEMP_LEAVES,
        &transaction,
    );
    let mut store_smt = CotaSMT::new(H256::zero(), smt_store);
    store_smt
        .update_all(update_leaves.clone())
        .expect("smt update leaves error");
    store_smt
        .merkle_proof(update_leaves.iter().map(|leave| leave.0).collect())
        .expect("smt proof error");
    let store_diff_time = (Local::now().timestamp_millis() - start_time) as f64 / 1000f64;

    println!(
        "History leaves count: {} and update leaves count: {}, default store smt: {} and rocksdb store smt: {}",
        history_leaf_count, update_leaf_count, default_diff_time, store_diff_time
    );

    assert!(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[bench]
    #[ignore]
    pub fn test_smt_with_5000_leaves(b: &mut Bencher) {
        b.iter(|| generate_smt(5000, 100));
    }

    #[bench]
    #[ignore]
    pub fn test_smt_with_10000_leaves(b: &mut Bencher) {
        b.iter(|| generate_smt(10000, 100));
    }

    #[bench]
    #[ignore]
    pub fn test_smt_with_50000_leaves(b: &mut Bencher) {
        b.iter(|| generate_smt(50000, 100));
    }
}
