//! The schema include constants define the low level database column families.

pub type Col = u8;
/// Total column number
pub const COLUMNS: u32 = 4;
/// Column SMT branch
pub const COLUMN_SMT_BRANCH: Col = 0;
/// Column SMT leaf
pub const COLUMN_SMT_LEAF: Col = 1;
/// Column SMT root hash
pub const COLUMN_SMT_ROOT: Col = 2;
/// Column SMT temp leaves
pub const COLUMN_SMT_TEMP_LEAVES: Col = 3;
