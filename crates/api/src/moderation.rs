use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use zeroclaw_core::repositories::{moderation, users};

use crate::error::AppError;
use crate::extractors::AuthUser;
use crate::state::AppState;

const MAX_REPORT_REASON_CHARS: usize = 2_000;

#[derive(Debug, Deserialize)]
pub struct ReportRequest {
    target_kind: String,
    target_id: Uuid,
    reason: String,
}

#[derive(Debug, Serialize)]
pub struct BlockResponse {
    blocker_id: String,
    blocked_id: String,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ReportResponse {
    id: String,
    reporter_id: String,
    target_kind: &'static str,
    target_id: String,
    reason: String,
    status: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

pub async fn block_user(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(handle): Path<String>,
) -> Result<(StatusCode, Json<BlockResponse>), AppError> {
    let Some(user) = users::find_by_handle(state.db_pool(), &handle).await? else {
        return Err(AppError::NotFound);
    };
    if user.id() == auth_user.id() {
        return Err(AppError::BadRequest("blocked_user"));
    }

    let block = moderation::block_user(state.db_pool(), auth_user.id(), user.id()).await?;

    Ok((StatusCode::CREATED, Json(BlockResponse::from(block))))
}

pub async fn unblock_user(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(handle): Path<String>,
) -> Result<StatusCode, AppError> {
    let Some(user) = users::find_by_handle(state.db_pool(), &handle).await? else {
        return Err(AppError::NotFound);
    };

    moderation::unblock_user(state.db_pool(), auth_user.id(), user.id()).await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn create_report(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<ReportRequest>,
) -> Result<(StatusCode, Json<ReportResponse>), AppError> {
    let target_kind = parse_target_kind(&payload.target_kind)?;
    let reason = validate_reason(payload.reason)?;
    let report = moderation::create_report(
        state.db_pool(),
        auth_user.id(),
        target_kind,
        payload.target_id,
        &reason,
    )
    .await?;

    Ok((StatusCode::CREATED, Json(ReportResponse::from(report))))
}

fn parse_target_kind(value: &str) -> Result<moderation::ReportTargetKind, AppError> {
    let value = value.trim().to_ascii_lowercase();
    moderation::ReportTargetKind::from_str(&value).ok_or(AppError::BadRequest("target_kind"))
}

fn validate_reason(reason: String) -> Result<String, AppError> {
    let reason = reason.trim().to_owned();
    if reason.is_empty() || reason.chars().count() > MAX_REPORT_REASON_CHARS {
        return Err(AppError::BadRequest("reason"));
    }

    Ok(reason)
}

impl From<moderation::Block> for BlockResponse {
    fn from(block: moderation::Block) -> Self {
        Self {
            blocker_id: block.blocker_id.to_string(),
            blocked_id: block.blocked_id.to_string(),
            created_at: block.created_at,
        }
    }
}

impl From<moderation::Report> for ReportResponse {
    fn from(report: moderation::Report) -> Self {
        Self {
            id: report.id.to_string(),
            reporter_id: report.reporter_id.to_string(),
            target_kind: report.target_kind.as_str(),
            target_id: report.target_id.to_string(),
            reason: report.reason,
            status: report.status,
            created_at: report.created_at,
            updated_at: report.updated_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_reason_must_be_present_and_bounded() {
        assert!(matches!(
            validate_reason("   ".to_owned()),
            Err(AppError::BadRequest("reason"))
        ));
        assert!(validate_reason("spam".to_owned()).is_ok());
    }

    #[test]
    fn report_target_kind_accepts_known_values() {
        assert!(matches!(
            parse_target_kind("post"),
            Ok(moderation::ReportTargetKind::Post)
        ));
        assert!(matches!(
            parse_target_kind("bad"),
            Err(AppError::BadRequest("target_kind"))
        ));
    }
}
