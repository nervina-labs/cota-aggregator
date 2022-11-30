use crate::{utils::error::Error, POOL};
use diesel::mysql::MysqlConnection;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};

pub(crate) mod block;
pub(crate) mod claim;
pub(crate) mod class;
pub(crate) mod common;
pub(crate) mod define;
pub(crate) mod extension;
pub mod helper;
pub(crate) mod hold;
pub(crate) mod issuer;
pub(crate) mod joyid;
pub(crate) mod scripts;
pub(crate) mod withdrawal;

type DBResult<T> = Result<(Vec<T>, u64), Error>;
type DBTotalResult<T> = Result<(Vec<T>, i64, u64), Error>;

pub type SqlConnectionPool = Pool<ConnectionManager<MysqlConnection>>;
pub type SqlPooledConnection = PooledConnection<ConnectionManager<MysqlConnection>>;

pub fn get_conn() -> SqlPooledConnection {
    POOL.clone().get().expect("Mysql pool connection error")
}
