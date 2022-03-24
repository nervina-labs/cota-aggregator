use super::schema::Col;
use crate::smt::db::schema::COLUMNS;
use crate::utils::error::Error;
use crate::utils::error::Error::RocksDBError;
pub use rocksdb::DBPinnableSlice;
use rocksdb::{ColumnFamily, ColumnFamilyDescriptor, Options, SliceTransform, WriteBatch, DB};
use serde_json::to_vec;
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
        opts.set_prefix_extractor(SliceTransform::create_fixed_prefix(36));

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

    pub fn get_with_prefix(&self, prefix: &[u8]) -> Vec<Vec<u8>> {
        let key_values = self.inner.prefix_iterator(prefix);
        let mut values: Vec<Vec<u8>> = vec![];
        for (_, value) in key_values {
            values.push(value.to_vec());
        }
        values
    }

    pub fn put(&self, col: Col, key: &[u8], value: &[u8]) -> Result<(), Error> {
        let cf = self.cf_handle(col)?;
        self.inner
            .put_cf(cf, key, value)
            .map_err(|_e| RocksDBError("RocksDB put error".to_string()))
    }

    pub fn batch_write(&self, key_values: Vec<(Vec<u8>, Vec<u8>)>) -> Result<(), Error> {
        let mut batch = WriteBatch::default();
        for (key, value) in key_values {
            batch.put(key, value);
        }
        self.inner
            .write(batch)
            .map_err(|_e| RocksDBError("RocksDB put error".to_string()))
    }

    pub fn delete(&self, col: Col, key: &[u8]) -> Result<(), Error> {
        let cf = self.cf_handle(col)?;
        self.inner
            .delete_cf(cf, key)
            .map_err(|_e| RocksDBError("RocksDB delete error".to_string()))
    }

    pub fn batch_delete(&self, keys: Vec<Vec<u8>>) -> Result<(), Error> {
        for key in keys {
            if self.inner.delete(key).is_err() {
                return Err(RocksDBError("RocksDB delete error".to_string()));
            }
        }
        Ok(())
    }
}
