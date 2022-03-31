use super::schema::Col;
use crate::smt::db::schema::COLUMNS;
use crate::smt::db::transaction::RocksDBTransaction;
use crate::utils::error::Error;
use rocksdb::ops::{GetColumnFamilys, OpenCF};
use rocksdb::{
    ColumnFamily, ColumnFamilyDescriptor, OptimisticTransactionDB, OptimisticTransactionOptions,
    Options, WriteOptions,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct RocksDB {
    pub(crate) inner: Arc<OptimisticTransactionDB>,
}

impl RocksDB {
    pub fn default() -> Result<Self, Error> {
        Self::new_with_path("./store.db")
    }

    pub fn new_with_path(path: &str) -> Result<Self, Error> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);

        let cf_names: Vec<_> = (0..COLUMNS).map(|c| c.to_string()).collect();
        let cf_descriptors: Vec<_> = cf_names
            .iter()
            .map(|c| ColumnFamilyDescriptor::new(c, Options::default()))
            .collect();

        let db = OptimisticTransactionDB::open_cf_descriptors(&opts, path, cf_descriptors)
            .map_err(|e| Error::RocksDBError(format!("RocksDB open error: {:?}", e.to_string())))?;
        Ok(RocksDB {
            inner: Arc::new(db),
        })
    }

    pub fn transaction(&self) -> RocksDBTransaction {
        let write_options = WriteOptions::default();
        let transaction_options = OptimisticTransactionOptions::new();

        RocksDBTransaction {
            db:    Arc::clone(&self.inner),
            inner: self.inner.transaction(&write_options, &transaction_options),
        }
    }

    pub fn inner(&self) -> Arc<OptimisticTransactionDB> {
        Arc::clone(&self.inner)
    }
}

#[inline]
pub(crate) fn cf_handle(db: &OptimisticTransactionDB, col: Col) -> Result<&ColumnFamily, Error> {
    db.cf_handle(&col.to_string())
        .ok_or_else(|| Error::RocksDBError(format!("column {} not found", col)))
}
