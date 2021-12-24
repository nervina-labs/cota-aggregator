use failure::Fail;

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

    #[fail(display = "Parse hex error")]
    ParseHexError,

    #[fail(display = "The cota_id '{}' has not defined", _0)]
    CotaIdHasNotDefined(String),

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
            Self::ParseHexError => "Parse hex error".to_string(),
            Self::CotaIdHasNotDefined(msg) => format!("The cota_id '{}' has not defined", msg),
            Self::Other(msg) => format!("Other error: {}", msg),
        }
    }
}
