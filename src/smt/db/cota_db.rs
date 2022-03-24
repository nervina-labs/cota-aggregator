use crate::smt::db::db::RocksDB;
use crate::smt::db::schema::Col;
use crate::utils::error::Error;

pub struct CotaRocksDB {
    pub(crate) inner: RocksDB,
}

impl CotaRocksDB {
    pub fn default() -> Self {
        CotaRocksDB {
            inner: RocksDB::default().expect("RocksDB create error"),
        }
    }

    pub fn get(&self, col: Col, key: &[u8]) -> Option<Box<[u8]>> {
        self.inner
            .get(col, key)
            .expect("db operation should be ok")
            .map(|v| Box::<[u8]>::from(v.as_ref()))
    }

    pub fn insert_raw(&self, col: Col, key: &[u8], value: &[u8]) -> Result<(), Error> {
        self.inner.put(col, key, value)
    }

    pub fn delete(&self, col: Col, key: &[u8]) -> Result<(), Error> {
        self.inner.delete(col, key)
    }

    pub fn batch_write(&self, key_values: Vec<(Vec<u8>, Vec<u8>)>) -> Result<(), Error> {
        self.inner.batch_write(key_values)
    }

    pub fn batch_delete(&self, keys: Vec<Vec<u8>>) -> Result<(), Error> {
        self.inner.batch_delete(keys)
    }

    pub fn get_with_prefix(&self, prefix: &[u8]) -> Vec<Vec<u8>> {
        self.inner.get_with_prefix(prefix)
    }
}
