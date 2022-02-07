#[derive(Queryable)]
pub struct Scripts {
    pub id: i32,
    pub code_hash: String,
    pub code_hash_crc: u32,
    pub hash_type: u8,
    pub args: String,
    pub args_crc: u32,
    pub created_at: String,
    pub updated_at: String,
}