// smt type
pub const DEFINE_NFT_SMT_TYPE: [u8; 2] = [129u8, 0u8]; // 0x8100
pub const HOLD_NFT_SMT_TYPE: [u8; 2] = [129u8, 1u8]; // 0x8101
pub const WITHDRAWAL_NFT_SMT_TYPE: [u8; 2] = [129u8, 2u8]; // 0x8102
pub const CLAIM_NFT_SMT_TYPE: [u8; 2] = [129u8, 3u8]; // 0x8103

// block height
pub const BLOCK_HEIGHT_VALUE_PADDING_MAINNET: u64 = 7220728;
pub const BLOCK_HEIGHT_VALUE_PADDING_TESTNET: u64 = 5466881;
