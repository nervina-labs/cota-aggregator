use crate::smt::store::smt_store::SMTStore;
use cota_smt::smt::{Blake2bHasher, H256};
use sparse_merkle_tree::SparseMerkleTree;

pub mod db;
pub mod store;
mod tests;
mod types;

pub type CotaSMT<'a> = SparseMerkleTree<Blake2bHasher, H256, SMTStore<'a>>;
