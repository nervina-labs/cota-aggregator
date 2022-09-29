use lazy_static::lazy_static;
use parking_lot::{Condvar, Mutex};
use std::collections::HashSet;
use std::sync::Arc;

pub(crate) mod claim;
pub(crate) mod claim_update;
mod constants;
pub(crate) mod define;
pub(crate) mod extension;
pub mod helper;
pub(crate) mod mint;
pub mod smt;
pub(crate) mod transfer;
pub(crate) mod transfer_update;
pub(crate) mod update;
pub(crate) mod withdrawal;
mod witness;

lazy_static! {
    static ref SMT_LOCK: Arc<(Mutex<HashSet<[u8; 32]>>, Condvar)> =
        Arc::new((Mutex::new(HashSet::new()), Condvar::new()));
}
