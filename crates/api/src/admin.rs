use axum::extract::{Path, Query, State};
use axum::Json;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use zeroclaw_core::models::UserId;
use zeroclaw_core::repositories::{comments, moderation, posts, users};

use crate::error::AppError;
use crate::extractors::AuthUser;
use crate::moderation::ReportResponse;
use crate::state::AppState;

const DEFAULT_REPORT_LIMIT: i64 = 50;
const MAX_REPORT_LIMIT: i64 = 100;
const MAX_AUDIT_NOTES_CHARS: usize = 2_000;

#[derive(Debug, Deserialize)]
pub struct ReportQueueQuery {
    status: Option<String>,
    limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct ReportActionRequest {
    action: String,
    notes: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ReportQueueResponse {
    reports: Vec<ReportResponse>,
}

#[derive(Debug, Serialize)]
pub struct AuditLogResponse {
    id: String,
    admin_id: String,
    report_id: Option<String>,
    action: String,
    target_kind: String,
    target_id: String,
    notes: Option<String>,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ReportActionResponse {
    report: ReportResponse,
    audit_log: AuditLogResponse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AdminReportAction {
    Dismiss,
    DeletePost,
    DeleteComment,
    SuspendUser,
}

impl AdminReportAction {
    fn parse(value: &str) -> Result<Self, AppError> {
        match value.trim() {
            "dismiss" => Ok(Self::Dismiss),
            "delete_post" => Ok(Self::DeletePost),
            "delete_comment" => Ok(Self::DeleteComment),
            "suspend_user" => Ok(Self::SuspendUser),
            _ => Err(AppError::BadRequest("action")),
        }
    }

    const fn as_str(self) -> &'static str {
        match self {
            Self::Dismiss => "dismiss",
            Self::DeletePost => "delete_post",
            Self::DeleteComment => "delete_comment",
            Self::SuspendUser => "suspend_user",
        }
    }

    const fn report_status(self) -> &'static str {
        match self {
            Self::Dismiss => "dismissed",
            Self::DeletePost | Self::DeleteComment | Self::SuspendUser => "actioned",
        }
    }
}

pub async fn list_pending_reports(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Query(query): Query<ReportQueueQuery>,
) -> Result<Json<ReportQueueResponse>, AppError> {
    ensure_admin(&auth_user)?;
    let status = query.status.unwrap_or_else(|| "open".to_owned());
    validate_report_status(&status)?;
    let limit = query
        .limit
        .unwrap_or(DEFAULT_REPORT_LIMIT)
        .clamp(1, MAX_REPORT_LIMIT);
    let reports = moderation::list_reports_by_status(state.db_pool(), &status, limit).await?;

    Ok(Json(ReportQueueResponse {
        reports: reports.into_iter().map(ReportResponse::from).collect(),
    }))
}

pub async fn take_report_action(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<ReportActionRequest>,
) -> Result<Json<ReportActionResponse>, AppError> {
    ensure_admin(&auth_user)?;
    let action = AdminReportAction::parse(&payload.action)?;
    let notes = validate_notes(payload.notes)?;
    let Some(report) = moderation::find_report(state.db_pool(), id).await? else {
        return Err(AppError::NotFound);
    };

    perform_action(&state, action, &report).await?;
    let updated_report =
        moderation::update_report_status(state.db_pool(), report.id, action.report_status()).await?;
    let audit_log = moderation::create_audit_log(
        state.db_pool(),
        auth_user.id(),
        Some(report.id),
        action.as_str(),
        report.target_kind.as_str(),
        report.target_id,
        notes.as_deref(),
    )
    .await?;

    Ok(Json(ReportActionResponse {
        report: ReportResponse::from(updated_report),
        audit_log: AuditLogResponse::from(audit_log),
    }))
}

async fn perform_action(
    state: &AppState,
    action: AdminReportAction,
    report: &moderation::Report,
) -> Result<(), AppError> {
    match action {
        AdminReportAction::Dismiss => Ok(()),
        AdminReportAction::DeletePost => {
            if report.target_kind != moderation::ReportTargetKind::Post {
                return Err(AppError::BadRequest("target_kind"));
            }
            if !posts::admin_soft_delete(state.db_pool(), report.target_id).await? {
                return Err(AppError::NotFound);
            }
            Ok(())
        }
        AdminReportAction::DeleteComment => {
            if report.target_kind != moderation::ReportTargetKind::Comment {
                return Err(AppError::BadRequest("target_kind"));
            }
            if !comments::admin_soft_delete(state.db_pool(), report.target_id).await? {
                return Err(AppError::NotFound);
            }
            Ok(())
        }
        AdminReportAction::SuspendUser => {
            if report.target_kind != moderation::ReportTargetKind::User {
                return Err(AppError::BadRequest("target_kind"));
            }
            users::suspend(state.db_pool(), UserId::from(report.target_id)).await?;
            Ok(())
        }
    }
}

fn ensure_admin(auth_user: &AuthUser) -> Result<(), AppError> {
    if auth_user.user().is_admin() {
        Ok(())
    } else {
        Err(AppError::Unauthorized)
    }
}

fn validate_report_status(status: &str) -> Result<(), AppError> {
    match status {
        "open" | "reviewed" | "dismissed" | "actioned" => Ok(()),
        _ => Err(AppError::BadRequest("status")),
    }
}

fn validate_notes(notes: Option<String>) -> Result<Option<String>, AppError> {
    let notes = notes.map(|value| value.trim().to_owned());
    if notes
        .as_ref()
        .is_some_and(|value| value.chars().count() > MAX_AUDIT_NOTES_CHARS)
    {
        return Err(AppError::BadRequest("notes"));
    }

    Ok(notes.filter(|value| !value.is_empty()))
}

impl From<moderation::AuditLog> for AuditLogResponse {
    fn from(log: moderation::AuditLog) -> Self {
        Self {
            id: log.id.to_string(),
            admin_id: log.admin_id.to_string(),
            report_id: log.report_id.map(|id| id.to_string()),
            action: log.action,
            target_kind: log.target_kind,
            target_id: log.target_id.to_string(),
            notes: log.notes,
            created_at: log.created_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admin_report_action_parser_accepts_known_actions() {
        assert!(matches!(
            AdminReportAction::parse("delete_post"),
            Ok(AdminReportAction::DeletePost)
        ));
        assert!(matches!(
            AdminReportAction::parse("bad"),
            Err(AppError::BadRequest("action"))
        ));
    }

    #[test]
    fn audit_notes_are_trimmed_and_bounded() {
        assert_eq!(validate_notes(Some(" note ".to_owned())).unwrap(), Some("note".to_owned()));
        assert_eq!(validate_notes(Some("   ".to_owned())).unwrap(), None);
    }
}
