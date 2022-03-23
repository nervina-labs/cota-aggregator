use super::schema::Col;
use crate::smt::db::schema::COLUMNS;
use crate::utils::error::Error;
use crate::utils::error::Error::RocksDBError;
pub use rocksdb::DBPinnableSlice;
use rocksdb::{ColumnFamily, ColumnFamilyDescriptor, Options, DB};
use std::sync::Arc;

#[derive(Clone)]
pub struct RocksDB {
    pub(crate) inner: Arc<DB>,
}

impl RocksDB {
    pub fn default() -> Result<Self, Error> {
        let path = "./store.db";

        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);

        let cf_names: Vec<_> = (0..COLUMNS).map(|c| c.to_string()).collect();
        let cf_descriptors: Vec<_> = cf_names
            .iter()
            .map(|c| ColumnFamilyDescriptor::new(c, Options::default()))
            .collect();

        let db = DB::open_cf_descriptors(&opts, path, cf_descriptors)
            .map_err(|e| Error::RocksDBError(format!("RocksDB open error: {:?}", e.to_string())))?;
        Ok(RocksDB {
            inner: Arc::new(db),
        })
    }

    fn cf_handle(&self, col: Col) -> Result<&ColumnFamily, Error> {
        self.inner
            .cf_handle(&col.to_string())
            .ok_or_else(|| RocksDBError(format!("column {} not found", col)))
    }

    pub fn get(&self, col: Col, key: &[u8]) -> Result<Option<Vec<u8>>, Error> {
        let cf = self.cf_handle(col)?;
        self.inner
            .get_cf(cf, key)
            .map_err(|_e| RocksDBError("RocksDB get error".to_string()))
    }

    pub fn put(&self, col: Col, key: &[u8], value: &[u8]) -> Result<(), Error> {
        let cf = self.cf_handle(col)?;
        self.inner
            .put_cf(cf, key, value)
            .map_err(|_e| RocksDBError("RocksDB put error".to_string()))
    }

    pub fn delete(&self, col: Col, key: &[u8]) -> Result<(), Error> {
        let cf = self.cf_handle(col)?;
        self.inner
            .delete_cf(cf, key)
            .map_err(|_e| RocksDBError("RocksDB delete error".to_string()))
    }
}
