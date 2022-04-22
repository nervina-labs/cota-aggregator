use super::serde::{branch_key_to_vec, branch_node_to_vec, slice_to_branch_node};
use crate::smt::db::schema::Col;
use crate::smt::store::serde::leaf_key_to_vec;
use crate::smt::transaction::store_transaction::StoreTransaction;
use crate::smt::types::leaf::{Byte32, SMTLeaf, SMTLeafBuilder, SMTLeafVec, SMTLeafVecBuilder};
use crate::utils::error::Error;
use cota_smt::smt::H256;
use molecule::prelude::{Builder, Entity};
use sparse_merkle_tree::{
    error::Error as SMTError,
    traits::Store,
    tree::{BranchKey, BranchNode},
};
use std::convert::TryInto;

pub struct SMTStore<'a> {
    lock_hash:  [u8; 32],
    leaf_col:   Col,
    branch_col: Col,
    root_col:   Col,
    leaves_col: Col,
    store:      &'a StoreTransaction,
}

impl<'a> SMTStore<'a> {
    pub fn new(
        lock_hash: [u8; 32],
        leaf_col: Col,
        branch_col: Col,
        root_col: Col,
        leaves_col: Col,
        store: &'a StoreTransaction,
    ) -> Self {
        SMTStore {
            lock_hash,
            leaf_col,
            branch_col,
            root_col,
            leaves_col,
            store,
        }
    }

    pub fn save_root(&self, root: &H256) -> Result<(), Error> {
        self.store
            .insert_raw(self.root_col, &self.lock_hash, root.as_slice())
            .map_err(|err| Error::SMTError(format!("insert error {:?}", err)))?;

        Ok(())
    }

    pub fn get_root(&self) -> Result<Option<H256>, SMTError> {
        match self.store.get(self.root_col, &self.lock_hash) {
            Some(slice) => {
                let v: [u8; 32] = slice
                    .as_ref()
                    .try_into()
                    .expect("stored H256 should be valid");
                Ok(Some(v.into()))
            }
            None => Ok(None),
        }
    }

    pub fn insert_leaves(&self, leaves: Vec<(H256, H256)>) -> Result<(), Error> {
        let smt_leaf_vec: Vec<SMTLeaf> = leaves
            .into_iter()
            .map(|leaf| {
                SMTLeafBuilder::default()
                    .key(Byte32::from_slice(leaf.0.as_slice()).unwrap())
                    .value(Byte32::from_slice(leaf.1.as_slice()).unwrap())
                    .build()
            })
            .collect();
        let smt_leaves: SMTLeafVec = SMTLeafVecBuilder::default().set(smt_leaf_vec).build();
        self.store
            .insert_raw(self.leaves_col, &self.lock_hash, smt_leaves.as_slice())
            .map_err(|err| Error::SMTError(format!("insert error {:?}", err)))?;
        Ok(())
    }

    pub fn get_leaves(&self) -> Result<Option<Vec<(H256, H256)>>, Error> {
        match self.store.get(self.leaves_col, &self.lock_hash) {
            Some(slice) => {
                let smt_leaves = SMTLeafVec::from_slice(&slice)
                    .map_err(|_e| Error::SMTError("SMT Leaves parse error".to_owned()))?;
                let leaves = smt_leaves
                    .into_iter()
                    .map(|smt_leaf| {
                        let key: [u8; 32] = smt_leaf
                            .key()
                            .as_slice()
                            .try_into()
                            .expect("stored SMTLeaf should be valid");
                        let value: [u8; 32] = smt_leaf
                            .value()
                            .as_slice()
                            .try_into()
                            .expect("stored SMTLeaf should be valid");
                        (key.into(), value.into())
                    })
                    .collect();
                Ok(Some(leaves))
            }
            None => Ok(None),
        }
    }
}

impl<'a> Store<H256> for SMTStore<'a> {
    fn get_branch(&self, branch_key: &BranchKey) -> Result<Option<BranchNode>, SMTError> {
        match self.store.get(
            self.branch_col,
            &branch_key_to_vec(self.lock_hash, branch_key),
        ) {
            Some(slice) => Ok(Some(slice_to_branch_node(&slice))),
            None => Ok(None),
        }
    }

    fn get_leaf(&self, leaf_key: &H256) -> Result<Option<H256>, SMTError> {
        match self
            .store
            .get(self.leaf_col, &leaf_key_to_vec(self.lock_hash, leaf_key))
        {
            Some(slice) if 32 == slice.len() => {
                let mut leaf = [0u8; 32];
                leaf.copy_from_slice(slice.as_ref());
                Ok(Some(H256::from(leaf)))
            }
            Some(_) => Err(SMTError::Store("get corrupted leaf".to_string())),
            None => Ok(None),
        }
    }

    fn insert_branch(&mut self, branch_key: BranchKey, branch: BranchNode) -> Result<(), SMTError> {
        self.store
            .insert_raw(
                self.branch_col,
                &branch_key_to_vec(self.lock_hash, &branch_key),
                &branch_node_to_vec(&branch),
            )
            .map_err(|err| SMTError::Store(format!("insert error {:?}", err)))?;

        Ok(())
    }

    fn insert_leaf(&mut self, leaf_key: H256, leaf: H256) -> Result<(), SMTError> {
        self.store
            .insert_raw(
                self.leaf_col,
                &leaf_key_to_vec(self.lock_hash, &leaf_key),
                leaf.as_slice(),
            )
            .map_err(|err| SMTError::Store(format!("insert error {:?}", err)))?;

        Ok(())
    }

    fn remove_branch(&mut self, branch_key: &BranchKey) -> Result<(), SMTError> {
        self.store
            .delete(
                self.branch_col,
                &branch_key_to_vec(self.lock_hash, branch_key),
            )
            .map_err(|err| SMTError::Store(format!("delete error {:?}", err)))?;

        Ok(())
    }

    fn remove_leaf(&mut self, leaf_key: &H256) -> Result<(), SMTError> {
        self.store
            .delete(self.leaf_col, &leaf_key_to_vec(self.lock_hash, &leaf_key))
            .map_err(|err| SMTError::Store(format!("delete error {:?}", err)))?;

        Ok(())
    }
}
