/// Unified error type for the CHIPIN backend.
/// AppError converts into an Axum JSON response automatically.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::NotFound(msg)    => (StatusCode::NOT_FOUND,            msg.clone()),
            AppError::BadRequest(msg)  => (StatusCode::BAD_REQUEST,          msg.clone()),
            AppError::Database(e)      => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            AppError::Internal(msg)    => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
        };

        tracing::error!("Request error: {} — {}", status, message);

        (status, Json(json!({ "error": message }))).into_response()
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
