use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use thiserror::Error;
use zeroclaw_core::ConfigError;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("configuration error: {0}")]
    Config(#[from] ConfigError),

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

    #[error("tracing initialization error: {0}")]
    Tracing(String),
}

impl AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Database(_) | Self::Migration(_) | Self::Redis(_) => {
                StatusCode::SERVICE_UNAVAILABLE
            }
            Self::Config(_) | Self::Io(_) | Self::Server(_) | Self::Tracing(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }

    fn code(&self) -> &'static str {
        match self {
            Self::Config(_) => "configuration_error",
            Self::Database(_) => "database_error",
            Self::Migration(_) => "migration_error",
            Self::Redis(_) => "redis_error",
            Self::Io(_) => "io_error",
            Self::Server(_) => "server_error",
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
}
