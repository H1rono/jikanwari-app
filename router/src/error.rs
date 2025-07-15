pub struct Error {
    status: http::StatusCode,
    message: String,
}

impl Error {
    pub fn new(status: http::StatusCode, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
        }
    }
}

impl From<std::convert::Infallible> for Error {
    fn from(e: std::convert::Infallible) -> Self {
        match e {}
    }
}

impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        // TODO: tracing::error!(error = ?err, "An error occurred");
        Self::new(http::StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
    }
}

impl axum::response::IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        let Self { status, message } = self;
        (status, message).into_response()
    }
}
