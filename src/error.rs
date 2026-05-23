use thiserror::Error;

#[cfg(feature = "native")]
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Missing required parameter: {0}")]
    MissingParam(String),

    #[error("Invalid parameter '{param}': {reason}")]
    InvalidParam { param: &'static str, reason: String },

    #[error("GitHub API error: {0}")]
    GitHub(String),

    #[cfg(feature = "native")]
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
}

#[cfg(feature = "native")]
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::MissingParam(_) | AppError::InvalidParam { .. } => {
                (StatusCode::BAD_REQUEST, self.to_string())
            }
            AppError::GitHub(_) | AppError::Http(_) => {
                (StatusCode::BAD_GATEWAY, self.to_string())
            }
        };
        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}
