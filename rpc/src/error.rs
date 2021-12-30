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
        display = "Request parameter {} length, got {:x}, expected: {:x}",
        msg, got, expected
    )]
    RequestParamHexLenError {
        msg:      String,
        got:      usize,
        expected: usize,
    },

    #[fail(display = "Request parameter '{}' type error", _0)]
    RequestParamTypeError(String),

    #[fail(display = "The cota_id '{}' has not defined", _0)]
    CotaIdHasNotDefined(String),

    #[fail(display = "The cota_id and token_index has not withdrawn")]
    CotaIdAndTokenIndexHasNotWithdrawn,

    #[fail(display = "The cota_id and token_index has not held")]
    CotaIdAndTokenIndexHasNotHeld,

    #[fail(display = "Database '{}' query error", _0)]
    DatabaseQueryError(String),

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
                "Request parameter '{}' length, got {:x}, expected: {:x}",
                msg, got, expected
            ),
            Self::RequestParamTypeError(msg) => format!("Request parameter '{}' type error", msg),
            Self::CotaIdHasNotDefined(msg) => format!("The cota_id '{}' has not defined", msg),
            Self::CotaIdAndTokenIndexHasNotWithdrawn => {
                "The cota_id and token_index has not withdrawn".to_owned()
            }
            Self::CotaIdAndTokenIndexHasNotHeld => {
                "The cota_id and token_index has not held".to_owned()
            }
            Self::DatabaseQueryError(msg) => format!("Database '{}' query error", msg),
            Self::Other(msg) => format!("Other error: {}", msg),
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
