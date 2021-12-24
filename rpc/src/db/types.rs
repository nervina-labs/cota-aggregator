// Define
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct DefineDb {
    pub cota_id:   [u8; 20],
    pub total:     u32,
    pub issued:    u32,
    pub configure: u8,
}

// Mint
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct MintWithdrawal {
    pub token_index:    [u8; 4],
    pub configure:      u8,
    pub state:          u8,
    pub characteristic: [u8; 20],
    pub to_lock_hash:   [u8; 32],
}

#[derive(Clone, Eq, PartialEq)]
pub struct MintDb {
    pub lock_hash:   [u8; 32],
    pub cota_id:     [u8; 20],
    pub out_point:   [u8; 72],
    pub withdrawals: Vec<MintWithdrawal>,
}

// Withdraw
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Withdrawal {
    pub cota_id:      [u8; 20],
    pub token_index:  [u8; 4],
    pub to_lock_hash: [u8; 32],
}

#[derive(Clone, Eq, PartialEq)]
pub struct WithdrawDb {
    pub lock_hash:   [u8; 32],
    pub out_point:   [u8; 72],
    pub withdrawals: Vec<Withdrawal>,
}

// Claim
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Claim {
    pub cota_id:     [u8; 20],
    pub token_index: [u8; 4],
}

#[derive(Clone, Eq, PartialEq)]
pub struct ClaimDb {
    pub lock_hash: [u8; 32],
    pub claims:    Vec<Claim>,
}
