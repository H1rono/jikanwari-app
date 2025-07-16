use std::fmt;

#[derive(Debug)]
pub enum Error {
    NotFound(String),
    Unexpected(anyhow::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::NotFound(msg) => write!(f, "Not Found: {msg}"),
            Error::Unexpected(err) => write!(f, "Unexpected error: {err}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Error::Unexpected(err)
    }
}

impl repository::Error for Error {
    fn not_found(message: &str) -> Self {
        Error::NotFound(message.to_string())
    }
}

impl From<Error> for router::Error {
    fn from(err: Error) -> Self {
        match err {
            Error::NotFound(msg) => Self::new(http::StatusCode::NOT_FOUND, msg),
            Error::Unexpected(err) => err.into(),
        }
    }
}
