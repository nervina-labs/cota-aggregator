use crate::smt::store::smt_store::SMTStore;
use sparse_merkle_tree::blake2b::Blake2bHasher;
use sparse_merkle_tree::{SparseMerkleTree, H256};

pub mod db;
pub mod store;

pub type CotaSMT<'a> = SparseMerkleTree<Blake2bHasher, H256, SMTStore<'a>>;
