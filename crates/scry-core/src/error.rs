use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Plugin error: {0}")]
    Plugin(#[from] anyhow::Error),

    #[error("Authentication failed: {0}")]
    Auth(String),

    #[error("Invalid request: {0}")]
    BadRequest(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(#[from] validator::ValidationErrors),

    #[error("Internal server error")]
    Internal,
}

impl Error {
    pub fn code(&self) -> &'static str {
        match self {
            Error::Database(_) => "DATABASE_ERROR",
            Error::Plugin(_) => "PLUGIN_ERROR",
            Error::Auth(_) => "AUTH_FAILED",
            Error::BadRequest(_) => "BAD_REQUEST",
            Error::NotFound(_) => "NOT_FOUND",
            Error::Validation(_) => "VALIDATION_ERROR",
            Error::Internal => "INTERNAL_ERROR",
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let status = match self {
            Error::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Plugin(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Auth(_) => StatusCode::UNAUTHORIZED,
            Error::BadRequest(_) => StatusCode::BAD_REQUEST,
            Error::NotFound(_) => StatusCode::NOT_FOUND,
            Error::Validation(_) => StatusCode::BAD_REQUEST,
            Error::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        };

        // Zentrales Logging
        if status.is_server_error() {
            tracing::error!(error = ?self, code = self.code(), "Critical System Error");
        } else {
            tracing::warn!(error = %self, code = self.code(), "API Request Error");
        }

        let body = Json(json!({
            "error": self.to_string(),
            "code": self.code(),
        }));

        (status, body).into_response()
    }
}
