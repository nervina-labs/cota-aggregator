#[derive(Queryable)]
pub struct ClaimedCotaNft {
    pub id:            i32,
    pub block_number:  u64,
    pub cota_id:       String,
    pub cota_id_crc:   u32,
    pub token_index:   u32,
    pub out_point:     String,
    pub out_point_crc: u32,
    pub lock_hash:     String,
    pub lock_hash_crc: u32,
    pub created_at:    String,
    pub updated_at:    String,
}
