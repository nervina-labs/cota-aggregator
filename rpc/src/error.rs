use failure::Fail;

#[derive(Debug, Fail, Eq, PartialEq)]
pub enum Error {
    #[fail(display = "Request parameter [{}] parse error", _0)]
    RequestParamNotFound(String),

    #[fail(
        display = "Request parameter [{}] must be hex string starting with 0x",
        _0
    )]
    RequestParamHexInvalid(String),

    #[fail(
        display = "Request parameter length, got {:x}, expected: {:x}",
        got, expected
    )]
    RequestParamHexLenError { got: usize, expected: usize },

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
