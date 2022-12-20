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
    RequestParamHexLenError {
        msg:      String,
        got:      usize,
        expected: usize,
    },

    #[fail(display = "Request parameter '{}' type error", _0)]
    RequestParamTypeError(String),

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
    SocialFriendInfoError(String),

    #[fail(display = "CKB Script error")]
    CKBScriptError,

    #[fail(display = "Database '{}' query error", _0)]
    DatabaseQueryError(String),

    #[fail(display = "'{}' SMT proof error", _0)]
    SMTProofError(String),

    #[fail(display = "'{}' SMT error", _0)]
    SMTError(String),

    #[fail(display = "'{}' RocksDB error", _0)]
    RocksDBError(String),

    #[fail(display = "CKB Indexer error: {}", _0)]
    CKBIndexerError(String),

    #[fail(display = "CKB RPC error: {}", _0)]
    CKBRPCError(String),

    #[fail(display = "Witness Parse error: {}", _0)]
    WitnessParseError(String),

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
            Self::RequestParamHexLenError { msg, got, expected } => format!(
                "Request parameter '{}' length, got {}, expected: {}",
                msg, got, expected
            ),
            Self::RequestParamTypeError(msg) => format!("Request parameter '{}' type error", msg),
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
            Self::SocialFriendInfoError(msg) => {
                format!("The social friends information error: {}", msg)
            }
            Self::CKBScriptError => "CKB Script error".into(),
            Self::DatabaseQueryError(msg) => format!("Database query error: {}", msg),
            Self::SMTProofError(msg) => format!("'{}' SMT proof error", msg),
            Self::CKBIndexerError(msg) => format!("CKB Indexer error: {}", msg),
            Self::CKBRPCError(msg) => format!("CKB RPC error: {}", msg),
            Self::SMTError(msg) => format!("SMT error: {}", msg),
            Self::RocksDBError(msg) => format!("RocksDB error: {}", msg),
            Self::WitnessParseError(msg) => format!("Witness parse error: {}", msg),
            Self::Other(msg) => format!("Internal error: {:}", msg),
        }
    }
}

impl Into<RpcError> for Error {
    fn into(self) -> RpcError {
        RpcError {
            code:    ErrorCode::InvalidParams,
            message: self.to_msg(),
            data:    None,
        }
    }
}
