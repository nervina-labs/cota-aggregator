use failure::Fail;
use jsonrpc_http_server::jsonrpc_core::{Error as RpcError, ErrorCode};

#[derive(Debug, Fail, Eq, PartialEq)]
pub enum Error {
    #[fail(display = "Request parameter '{}' not found", _0)]
    RequestParamNotFound(String),

    #[fail(
        display = "Request parameter '{}' must be hex string starting with 0x",
        _0
    )]
    RequestParamHexInvalid(String),

    #[fail(
        display = "Request parameter {} length, got {}, expected: {}",
        msg, got, expected
    )]
    RequestParamHexLenInvalid {
        msg:      String,
        got:      usize,
        expected: usize,
    },

    #[fail(display = "Request parameter '{}' type error", _0)]
    RequestParamTypeInvalid(String),

    #[fail(display = "The collection of cota_id '{}' has not defined", _0)]
    CotaIdHasNotDefined(String),

    #[fail(display = "The NFT of cota_id and token_index has not been withdrawn")]
    CotaIdAndTokenIndexHasNotWithdrawn,

    #[fail(display = "The NFT of cota_id and token_index has not been held")]
    CotaIdAndTokenIndexHasNotHeld,

    #[fail(display = "The withdrawal CoTA NFTs are not in one transaction")]
    WithdrawCotaNFTsNotInOneTx,

    #[fail(display = "The subkey not found")]
    SubkeyLeafNotFound,

    #[fail(display = "The social recovery config not found")]
    SocialLeafNotFound,

    #[fail(display = "The social friends information '{}' error", _0)]
    SocialFriendInfoInvalid(String),

    #[fail(display = "CKB Script error")]
    CKBScriptInvalid,

    #[fail(display = "Database '{}' query error", _0)]
    DatabaseQueryInvalid(String),

    #[fail(display = "'{}' SMT proof error", _0)]
    SMTProofInvalid(String),

    #[fail(display = "'{}' SMT error", _0)]
    SMTInvalid(String),

    #[fail(display = "'{}' RocksDB error", _0)]
    RocksDBInvalid(String),

    #[fail(display = "CKB Indexer error: {}", _0)]
    CKBIndexerInvalid(String),

    #[fail(display = "CKB RPC error: {}", _0)]
    CKBRPCInvalid(String),

    #[fail(display = "Witness Parse error: {}", _0)]
    WitnessParseInvalid(String),

    #[fail(display = "Other error: {}", _0)]
    Other(String),
}

impl From<String> for Error {
    fn from(err: String) -> Error {
        Error::Other(err)
    }
}

impl From<&str> for Error {
    fn from(err: &str) -> Error {
        Error::Other(err.to_owned())
    }
}

impl Error {
    pub fn to_msg(self) -> String {
        match self {
            Self::RequestParamNotFound(msg) => format!("Request parameter '{}' not found", msg),
            Self::RequestParamHexInvalid(msg) => format!(
                "Request parameter '{}' must be hex string starting with 0x",
                msg
            ),
            Self::RequestParamHexLenInvalid { msg, got, expected } => format!(
                "Request parameter '{}' length, got {}, expected: {}",
                msg, got, expected
            ),
            Self::RequestParamTypeInvalid(msg) => format!("Request parameter '{}' type error", msg),
            Self::CotaIdHasNotDefined(msg) => format!("The cota_id '{}' has not defined", msg),
            Self::CotaIdAndTokenIndexHasNotWithdrawn => {
                "The cota_id and token_index has not withdrawn".into()
            }
            Self::CotaIdAndTokenIndexHasNotHeld => {
                "The cota_id and token_index has not held".into()
            }
            Self::WithdrawCotaNFTsNotInOneTx => {
                "The withdrawal CoTA NFTs are not in one transaction".into()
            }
            Self::SubkeyLeafNotFound => "The subkey not found".into(),
            Self::SocialLeafNotFound => "The social recovery config not found".into(),
            Self::SocialFriendInfoInvalid(msg) => {
                format!("The social friends information error: {}", msg)
            }
            Self::CKBScriptInvalid => "CKB Script error".into(),
            Self::DatabaseQueryInvalid(msg) => format!("Database query error: {}", msg),
            Self::SMTProofInvalid(msg) => format!("'{}' SMT proof error", msg),
            Self::CKBIndexerInvalid(msg) => format!("CKB Indexer error: {}", msg),
            Self::CKBRPCInvalid(msg) => format!("CKB RPC error: {}", msg),
            Self::SMTInvalid(msg) => format!("SMT error: {}", msg),
            Self::RocksDBInvalid(msg) => format!("RocksDB error: {}", msg),
            Self::WitnessParseInvalid(msg) => format!("Witness parse error: {}", msg),
            Self::Other(msg) => format!("Internal error: {:}", msg),
        }
    }
}

impl From<Error> for RpcError {
    fn from(val: Error) -> Self {
        RpcError {
            code:    ErrorCode::InvalidParams,
            message: val.to_msg(),
            data:    None,
        }
    }
}
