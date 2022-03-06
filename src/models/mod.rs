use crate::utils::error::Error;

pub(crate) mod block;
pub(crate) mod claim;
pub(crate) mod common;
pub(crate) mod define;
mod helper;
pub(crate) mod hold;
pub(crate) mod scripts;
pub(crate) mod withdrawal;

type DBResult<T> = Result<(Vec<T>, u64), Error>;
type DBTotalResult<T> = Result<(Vec<T>, i64, u64), Error>;
