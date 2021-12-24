#[derive(Copy, Clone, Eq, PartialEq)]
pub struct DefineDb {
    pub cota_id:   [u8; 20],
    pub total:     u32,
    pub issued:    u32,
    pub configure: u8,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct WithdrawDb {
    pub cota_id:            [u8; 20],
    pub token_index:        [u8; 4],
    pub out_point:          [u8; 72],
    pub state:              u8,
    pub configure:          u8,
    pub characteristic:     [u8; 20],
    pub receiver_lock_hash: [u8; 32],
}
