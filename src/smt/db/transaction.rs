use crate::smt::db::db::cf_handle;
use crate::smt::db::schema::Col;
use crate::utils::error::Error;
use rocksdb::ops::{DeleteCF, GetCF, IterateCF, PutCF};
use rocksdb::{DBVector, OptimisticTransaction, OptimisticTransactionDB};
use std::sync::Arc;

pub struct RocksDBTransaction {
    pub(crate) db:    Arc<OptimisticTransactionDB>,
    pub(crate) inner: OptimisticTransaction,
}

impl RocksDBTransaction {
    pub fn get(&self, col: Col, key: &[u8]) -> Result<Option<DBVector>, Error> {
        let cf = cf_handle(&self.db, col)?;
        self.inner
            .get_cf(cf, key)
            .map_err(|_e| Error::RocksDBError("transaction get_cf".to_owned()))
    }

    pub fn put(&self, col: Col, key: &[u8], value: &[u8]) -> Result<(), Error> {
        let cf = cf_handle(&self.db, col)?;
        self.inner
            .put_cf(cf, key, value)
            .map_err(|_e| Error::RocksDBError("transaction put_cf".to_owned()))
    }

    pub fn delete(&self, col: Col, key: &[u8]) -> Result<(), Error> {
        let cf = cf_handle(&self.db, col)?;
        self.inner
            .delete_cf(cf, key)
            .map_err(|_e| Error::RocksDBError("transaction delete_cf".to_owned()))
    }

    pub fn delete_batch(&self, col: Col, prefix: &[u8]) -> Result<(), Error> {
        let cf = cf_handle(&self.db, col)?;
        let mut iter = self
            .db
            .prefix_iterator_cf(cf, prefix)
            .map_err(|_e| Error::RocksDBError("transaction delete_cf".to_owned()))?;
        while iter.valid() {
            if let Some(kv) = iter.next() {
                self.delete(col, &kv.0)?;
            }
        }
        Ok(())
    }

    pub fn commit(&self) -> Result<(), Error> {
        self.inner
            .commit()
            .map_err(|e| Error::RocksDBError(format!("transaction commit: {:?}", e.to_string())))
    }

    pub fn rollback(&self) -> Result<(), Error> {
        self.inner
            .rollback()
            .map_err(|_e| Error::RocksDBError("transaction rollback".to_owned()))
    }
}
