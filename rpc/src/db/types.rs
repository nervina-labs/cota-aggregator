#[derive(Copy, Clone, Eq, PartialEq)]
pub struct DefineDb {
    pub cota_id:   [u8; 20],
    pub total:     u32,
    pub issued:    u32,
    pub configure: u8,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct HoldDb {
    pub cota_id:        [u8; 20],
    pub token_index:    [u8; 4],
    pub state:          u8,
    pub configure:      u8,
    pub characteristic: [u8; 20],
}

#[derive(Clone, Eq, PartialEq)]
pub struct WithdrawDb {
    pub cota_id:              [u8; 20],
    pub token_index:          [u8; 4],
    pub out_point:            [u8; 36],
    pub state:                u8,
    pub configure:            u8,
    pub characteristic:       [u8; 20],
    pub receiver_lock_script: Vec<u8>,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct ClaimDb {
    pub cota_id:     [u8; 20],
    pub token_index: [u8; 4],
    pub out_point:   [u8; 36],
}
