#[derive(Queryable)]
pub struct CheckInfos {
    pub id: i32,
    pub check_type: i8,
    pub block_number: u64,
    pub block_hash: String,
    pub created_at: String,
    pub updated_at: String,
}