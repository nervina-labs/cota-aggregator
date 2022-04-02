use crate::smt::store::smt_store::SMTStore;
use crate::utils::error::Error;
use cota_smt::smt::{Blake2bHasher, H256};
use log::debug;
use sparse_merkle_tree::SparseMerkleTree;

pub mod db;
pub mod store;
mod tests;
pub mod transaction;
mod types;

pub type CotaSMT<'a> = SparseMerkleTree<Blake2bHasher, H256, SMTStore<'a>>;

pub trait RootSaver {
    fn save_root_and_leaves(&self, leaves: Vec<(H256, H256)>) -> Result<(), Error>;
}

impl<'a> RootSaver for CotaSMT<'a> {
    fn save_root_and_leaves(&self, leaves: Vec<(H256, H256)>) -> Result<(), Error> {
        self.store()
            .save_root(self.root())
            .expect("Save smt root error");
        if !leaves.is_empty() {
            self.store().insert_leaves(leaves)?;
            self.store().commit()?;
        }
        debug!("Save latest smt root: {:?} and leaves", self.root());
        Ok(())
    }
}
