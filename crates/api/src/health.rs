use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::Serialize;
use zeroclaw_core::{db, redis};

use crate::state::AppState;

pub async fn healthz(State(state): State<AppState>) -> (StatusCode, Json<HealthResponse>) {
    let database = match db::health_check(state.db_pool()).await {
        Ok(()) => ComponentHealth::ok(),
        Err(error) => ComponentHealth::unavailable(error.to_string()),
    };

    let mut redis_manager = state.redis_manager();
    let redis = match redis::health_check(&mut redis_manager).await {
        Ok(()) => ComponentHealth::ok(),
        Err(error) => ComponentHealth::unavailable(error.to_string()),
    };

    let status = if database.available && redis.available {
        ServiceStatus::Ok
    } else {
        ServiceStatus::Degraded
    };

    let status_code = match status {
        ServiceStatus::Ok => StatusCode::OK,
        ServiceStatus::Degraded => StatusCode::SERVICE_UNAVAILABLE,
    };

    (
        status_code,
        Json(HealthResponse {
            status,
            checks: HealthChecks { database, redis },
        }),
    )
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceStatus {
    Ok,
    Degraded,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    status: ServiceStatus,
    checks: HealthChecks,
}

#[derive(Debug, Serialize)]
pub struct HealthChecks {
    database: ComponentHealth,
    redis: ComponentHealth,
}

#[derive(Debug, Serialize)]
pub struct ComponentHealth {
    available: bool,
    error: Option<String>,
}

impl ComponentHealth {
    fn ok() -> Self {
        Self {
            available: true,
            error: None,
        }
    }

    fn unavailable(error: String) -> Self {
        Self {
            available: false,
            error: Some(error),
        }
    }
}
