use std::fmt;

#[derive(Debug)]
pub enum Error {
    NotFound(String),
    Unauthenticated(String),
    Forbidden(String),
    Unexpected(anyhow::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::NotFound(msg) => write!(f, "Not Found: {msg}"),
            Error::Unauthenticated(msg) => write!(f, "Unauthenticated: {msg}"),
            Error::Forbidden(msg) => write!(f, "Forbidden: {msg}"),
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

impl authz::Error for Error {}

impl repository::Error for Error {
    fn not_found(message: &str) -> Self {
        Error::NotFound(message.to_string())
    }
}

impl service::Error for Error {
    fn unauthenticated(message: &str) -> Self {
        Error::Unauthenticated(message.to_string())
    }

    fn forbidden(message: &str) -> Self {
        Error::Forbidden(message.to_string())
    }
}

impl From<Error> for router::Error {
    fn from(err: Error) -> Self {
        match err {
            Error::NotFound(msg) => Self::new(http::StatusCode::NOT_FOUND, msg),
            Error::Unauthenticated(msg) => Self::new(http::StatusCode::UNAUTHORIZED, msg),
            Error::Forbidden(msg) => Self::new(http::StatusCode::FORBIDDEN, msg),
            Error::Unexpected(err) => err.into(),
        }
    }
}
