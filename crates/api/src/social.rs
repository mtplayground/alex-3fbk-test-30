use axum::extract::{Path, State};
use axum::Json;
use redis::AsyncCommands;
use serde::Serialize;
use std::time::Duration;
use uuid::Uuid;
use zeroclaw_core::repositories::social::{self, LikeTargetKind};

use crate::error::AppError;
use crate::extractors::AuthUser;
use crate::notifications;
use crate::state::AppState;

const COUNT_CACHE_TTL_SECONDS: u64 = 60 * 10;
const COUNT_RECONCILIATION_INTERVAL_SECONDS: u64 = 60 * 5;

#[derive(Debug, Serialize)]
pub struct ToggleCountResponse {
    active: bool,
    count: i64,
}

pub async fn toggle_post_like(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<ToggleCountResponse>, AppError> {
    let result = social::toggle_like(state.db_pool(), auth_user.id(), LikeTargetKind::Post, id)
        .await
        .map_err(map_toggle_error)?;
    cache_like_count(&state, LikeTargetKind::Post, id, result.count).await?;
    if result.active {
        notifications::emit_like(&state, auth_user.id(), LikeTargetKind::Post, id).await;
    }

    Ok(Json(ToggleCountResponse::from(result)))
}

pub async fn toggle_comment_like(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<ToggleCountResponse>, AppError> {
    let result = social::toggle_like(state.db_pool(), auth_user.id(), LikeTargetKind::Comment, id)
        .await
        .map_err(map_toggle_error)?;
    cache_like_count(&state, LikeTargetKind::Comment, id, result.count).await?;
    if result.active {
        notifications::emit_like(&state, auth_user.id(), LikeTargetKind::Comment, id).await;
    }

    Ok(Json(ToggleCountResponse::from(result)))
}

pub async fn toggle_post_save(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<ToggleCountResponse>, AppError> {
    let result = social::toggle_save(state.db_pool(), auth_user.id(), id)
        .await
        .map_err(map_toggle_error)?;
    cache_save_count(&state, id, result.count).await?;

    Ok(Json(ToggleCountResponse::from(result)))
}

pub fn spawn_count_reconciliation(state: AppState) {
    tokio::spawn(async move {
        let mut interval =
            tokio::time::interval(Duration::from_secs(COUNT_RECONCILIATION_INTERVAL_SECONDS));
        interval.tick().await;

        loop {
            interval.tick().await;
            if let Err(error) = reconcile_count_cache(&state).await {
                tracing::warn!(error = %error, "social count cache reconciliation failed");
            }
        }
    });
}

impl From<social::ToggleResult> for ToggleCountResponse {
    fn from(result: social::ToggleResult) -> Self {
        Self {
            active: result.active,
            count: result.count,
        }
    }
}

async fn cache_like_count(
    state: &AppState,
    target_kind: LikeTargetKind,
    target_id: Uuid,
    count: i64,
) -> Result<(), AppError> {
    let key = state.redis_namespace().key([
        "counts".to_owned(),
        "likes".to_owned(),
        target_kind.as_str().to_owned(),
        target_id.to_string(),
    ]);
    cache_count(state, key, count).await
}

async fn cache_save_count(state: &AppState, post_id: Uuid, count: i64) -> Result<(), AppError> {
    let key = state.redis_namespace().key([
        "counts".to_owned(),
        "saves".to_owned(),
        "post".to_owned(),
        post_id.to_string(),
    ]);
    cache_count(state, key, count).await
}

async fn cache_count(state: &AppState, key: String, count: i64) -> Result<(), AppError> {
    let mut redis = state.redis_manager();
    redis
        .set_ex::<_, _, ()>(key, count, COUNT_CACHE_TTL_SECONDS)
        .await?;

    Ok(())
}

async fn reconcile_count_cache(state: &AppState) -> Result<(), AppError> {
    for count in social::all_like_counts(state.db_pool()).await? {
        let target_kind = match count.target_kind.as_str() {
            "post" => LikeTargetKind::Post,
            "comment" => LikeTargetKind::Comment,
            _ => continue,
        };
        cache_like_count(state, target_kind, count.target_id, count.count).await?;
    }

    for count in social::all_save_counts(state.db_pool()).await? {
        cache_save_count(state, count.post_id, count.count).await?;
    }

    Ok(())
}

fn map_toggle_error(error: sqlx::Error) -> AppError {
    match error {
        sqlx::Error::RowNotFound => AppError::NotFound,
        other => AppError::Database(other),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggle_response_maps_active_and_count() {
        let response = ToggleCountResponse::from(social::ToggleResult {
            active: true,
            count: 7,
        });

        assert!(response.active);
        assert_eq!(response.count, 7);
    }
}
