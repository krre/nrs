use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Unauthorized(String),
    #[error("{0}")]
    NotFound(String),
    #[error("Resource already exists")]
    Conflict,
    #[error("{0}")]
    InternalServerError(String),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::Unauthorized(err) => (StatusCode::UNAUTHORIZED, err),
            Self::NotFound(err) => (StatusCode::NOT_FOUND, err),
            Self::Conflict => (StatusCode::CONFLICT, Self::Conflict.to_string()),
            Self::InternalServerError(err) => (StatusCode::INTERNAL_SERVER_ERROR, err),
        };

        let body = Json(message);

        (status, body).into_response()
    }
}
