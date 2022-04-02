use crate::smt::db::schema::Col;
use crate::smt::db::transaction::RocksDBTransaction;
use crate::utils::error::Error;

pub struct StoreTransaction {
    pub(crate) inner: RocksDBTransaction,
}

impl StoreTransaction {
    pub fn new(inner: RocksDBTransaction) -> Self {
        StoreTransaction { inner }
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

    pub fn commit(&self) -> Result<(), Error> {
        self.inner.commit()
    }
}
