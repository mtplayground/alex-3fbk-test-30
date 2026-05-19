use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use thiserror::Error;
use zeroclaw_core::{auth::AuthError, storage::StorageError, ConfigError};

use crate::email::EmailError;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("bad request: {0}")]
    BadRequest(&'static str),

    #[error("unauthorized")]
    Unauthorized,

    #[error("conflict: {0}")]
    Conflict(&'static str),

    #[error("not found")]
    NotFound,

    #[error("rate limit exceeded")]
    RateLimited,

    #[error("configuration error: {0}")]
    Config(#[from] ConfigError),

    #[error("authentication error: {0}")]
    Auth(#[from] AuthError),

    #[error("email error: {0}")]
    Email(#[from] EmailError),

    #[error("storage error: {0}")]
    Storage(#[from] StorageError),

    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("database migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),

    #[error("redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("server error: {0}")]
    Server(#[from] axum::Error),

    #[error("internal error: {0}")]
    Internal(String),

    #[error("tracing initialization error: {0}")]
    Tracing(String),
}

impl AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::RateLimited => StatusCode::TOO_MANY_REQUESTS,
            Self::Database(_) | Self::Migration(_) | Self::Redis(_) => {
                StatusCode::SERVICE_UNAVAILABLE
            }
            Self::Config(_)
            | Self::Auth(_)
            | Self::Email(_)
            | Self::Storage(_)
            | Self::Io(_)
            | Self::Server(_)
            | Self::Internal(_)
            | Self::Tracing(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn code(&self) -> &'static str {
        match self {
            Self::BadRequest(_) => "bad_request",
            Self::Unauthorized => "unauthorized",
            Self::Conflict(_) => "conflict",
            Self::NotFound => "not_found",
            Self::RateLimited => "rate_limited",
            Self::Config(_) => "configuration_error",
            Self::Auth(_) => "authentication_error",
            Self::Email(_) => "email_error",
            Self::Storage(_) => "storage_error",
            Self::Database(_) => "database_error",
            Self::Migration(_) => "migration_error",
            Self::Redis(_) => "redis_error",
            Self::Io(_) => "io_error",
            Self::Server(_) => "server_error",
            Self::Internal(_) => "internal_error",
            Self::Tracing(_) => "tracing_error",
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let body = ErrorResponse {
            error: ErrorBody {
                code: self.code(),
                message: self.to_string(),
            },
        };

        (status, Json(body)).into_response()
    }
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: ErrorBody,
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    code: &'static str,
    message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redis_errors_map_to_service_unavailable() {
        let error = AppError::Redis(redis::RedisError::from((
            redis::ErrorKind::IoError,
            "redis unavailable",
        )));

        assert_eq!(error.status_code(), StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(error.code(), "redis_error");
    }

    #[test]
    fn unauthorized_errors_map_to_unauthorized() {
        let error = AppError::Unauthorized;

        assert_eq!(error.status_code(), StatusCode::UNAUTHORIZED);
        assert_eq!(error.code(), "unauthorized");
    }

    #[test]
    fn rate_limited_errors_map_to_too_many_requests() {
        let error = AppError::RateLimited;

        assert_eq!(error.status_code(), StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(error.code(), "rate_limited");
    }
}
