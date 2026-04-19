use axum::{Json, http::StatusCode, response::IntoResponse};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    BadRequest(String),
    #[error("Forbidden")]
    Forbidden,
    #[error("Internal server error")]
    Internal,
    #[error("Not found")]
    NotFound,
    #[error("Too many requests")]
    TooManyRequests,
    #[error("{0}")]
    Unauthorized(String),
    #[error("Unsupported media type")]
    UnsupportedMediaType,
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            Error::BadRequest(message) => (StatusCode::BAD_REQUEST, message),
            Error::Forbidden => (StatusCode::FORBIDDEN, self.to_string()),
            Error::Internal => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            Error::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
            Error::TooManyRequests => (StatusCode::TOO_MANY_REQUESTS, self.to_string()),
            Error::Unauthorized(message) => (StatusCode::UNAUTHORIZED, message),
            Error::UnsupportedMediaType => (StatusCode::UNSUPPORTED_MEDIA_TYPE, self.to_string()),
        };

        let body = Json(json!({
            "message": message
        }));

        (status, body).into_response()
    }
}
