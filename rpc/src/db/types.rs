#[derive(Copy, Clone, Eq, PartialEq)]
pub struct DefineDb {
    pub cota_id:   [u8; 32],
    pub total:     u32,
    pub issued:    u32,
    pub configure: u8,
}
