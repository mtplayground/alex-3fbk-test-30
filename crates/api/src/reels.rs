use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use zeroclaw_core::repositories::reels;

use crate::error::AppError;
use crate::extractors::AuthUser;
use crate::state::AppState;

const DEFAULT_REELS_LIMIT: i64 = 10;
const MAX_REELS_LIMIT: i64 = 30;
const MAX_REEL_CAPTION_LENGTH: usize = 2_200;
const MAX_AUDIO_FIELD_LENGTH: usize = 140;

#[derive(Debug, Deserialize)]
pub struct CreateReelRequest {
    media_id: Uuid,
    caption: Option<String>,
    duration_ms: Option<i64>,
    audio_title: Option<String>,
    audio_artist: Option<String>,
    audio_is_original: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ReelsQuery {
    cursor: Option<String>,
    limit: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReelResponse {
    id: String,
    author: ReelAuthorResponse,
    caption: String,
    media: ReelMediaResponse,
    duration_ms: Option<i64>,
    audio: ReelAudioResponse,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReelAuthorResponse {
    id: String,
    handle: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReelMediaResponse {
    media_id: String,
    kind: String,
    status: String,
    original_key: String,
    variants: serde_json::Value,
    width: Option<i32>,
    height: Option<i32>,
    duration_ms: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReelAudioResponse {
    title: Option<String>,
    artist: Option<String>,
    is_original: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReelsPageResponse {
    reels: Vec<ReelResponse>,
    next_cursor: Option<String>,
}

pub async fn create_reel(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<CreateReelRequest>,
) -> Result<(StatusCode, Json<ReelResponse>), AppError> {
    let caption = normalize_caption(payload.caption)?;
    let audio_title = normalize_audio_field(payload.audio_title, "audio_title")?;
    let audio_artist = normalize_audio_field(payload.audio_artist, "audio_artist")?;
    validate_duration(payload.duration_ms)?;

    let input = reels::CreateReel {
        author_id: auth_user.id(),
        media_id: payload.media_id,
        caption,
        duration_ms: payload.duration_ms,
        audio_title,
        audio_artist,
        audio_is_original: payload.audio_is_original.unwrap_or(true),
    };
    let reel = reels::create(state.db_pool(), &input)
        .await
        .map_err(map_create_reel_error)?;

    Ok((StatusCode::CREATED, Json(ReelResponse::from(reel))))
}

pub async fn get_reels_feed(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Query(query): Query<ReelsQuery>,
) -> Result<Json<ReelsPageResponse>, AppError> {
    let limit = query
        .limit
        .unwrap_or(DEFAULT_REELS_LIMIT)
        .clamp(1, MAX_REELS_LIMIT);
    let cursor = parse_reel_cursor(query.cursor)?;
    let feed_reels = reels::list_feed(state.db_pool(), auth_user.id(), cursor, limit + 1).await?;
    let has_next = feed_reels.len() > limit as usize;
    let page_reels: Vec<_> = feed_reels.into_iter().take(limit as usize).collect();
    let next_cursor = if has_next {
        page_reels.last().map(reel_cursor)
    } else {
        None
    };

    Ok(Json(ReelsPageResponse {
        reels: page_reels
            .into_iter()
            .map(|feed_reel| ReelResponse::from(feed_reel.reel))
            .collect(),
        next_cursor,
    }))
}

pub async fn get_reel(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ReelResponse>, AppError> {
    let Some(reel) = reels::find_by_id(state.db_pool(), id).await? else {
        return Err(AppError::NotFound);
    };

    Ok(Json(ReelResponse::from(reel)))
}

impl From<reels::Reel> for ReelResponse {
    fn from(reel: reels::Reel) -> Self {
        Self {
            id: reel.id.to_string(),
            author: ReelAuthorResponse {
                id: reel.author_id.to_string(),
                handle: reel.author_handle,
            },
            caption: reel.caption,
            media: ReelMediaResponse::from(reel.media),
            duration_ms: reel.duration_ms,
            audio: ReelAudioResponse {
                title: reel.audio_title,
                artist: reel.audio_artist,
                is_original: reel.audio_is_original,
            },
            created_at: reel.created_at,
        }
    }
}

impl From<reels::ReelMedia> for ReelMediaResponse {
    fn from(media: reels::ReelMedia) -> Self {
        Self {
            media_id: media.media_id.to_string(),
            kind: media.kind,
            status: media.status,
            original_key: media.original_key,
            variants: media.variants,
            width: media.width,
            height: media.height,
            duration_ms: media.duration_ms,
        }
    }
}

fn normalize_caption(value: Option<String>) -> Result<String, AppError> {
    let caption = value.unwrap_or_default().trim().to_owned();
    if caption.chars().count() > MAX_REEL_CAPTION_LENGTH {
        return Err(AppError::BadRequest("caption"));
    }

    Ok(caption)
}

fn normalize_audio_field(
    value: Option<String>,
    field: &'static str,
) -> Result<Option<String>, AppError> {
    let Some(value) = value else {
        return Ok(None);
    };
    let value = value.trim().to_owned();
    if value.is_empty() {
        return Ok(None);
    }
    if value.chars().count() > MAX_AUDIO_FIELD_LENGTH {
        return Err(AppError::BadRequest(field));
    }

    Ok(Some(value))
}

fn validate_duration(value: Option<i64>) -> Result<(), AppError> {
    if matches!(value, Some(duration) if duration <= 0) {
        return Err(AppError::BadRequest("duration_ms"));
    }

    Ok(())
}

fn parse_reel_cursor(cursor: Option<String>) -> Result<Option<reels::FeedCursor>, AppError> {
    let Some(cursor) = cursor else {
        return Ok(None);
    };
    let cursor = cursor.trim();
    if cursor.is_empty() {
        return Ok(None);
    }

    let mut parts = cursor.split('|');
    let Some(rank_score) = parts.next() else {
        return Err(AppError::BadRequest("cursor"));
    };
    let Some(created_at) = parts.next() else {
        return Err(AppError::BadRequest("cursor"));
    };
    let Some(id) = parts.next() else {
        return Err(AppError::BadRequest("cursor"));
    };
    if parts.next().is_some() {
        return Err(AppError::BadRequest("cursor"));
    }

    let rank_score = rank_score
        .parse::<f64>()
        .map_err(|_| AppError::BadRequest("cursor"))?;
    if !rank_score.is_finite() {
        return Err(AppError::BadRequest("cursor"));
    }
    let created_at =
        DateTime::parse_from_rfc3339(created_at).map_err(|_| AppError::BadRequest("cursor"))?;
    let id = Uuid::parse_str(id).map_err(|_| AppError::BadRequest("cursor"))?;

    Ok(Some(reels::FeedCursor {
        rank_score,
        created_at: created_at.with_timezone(&Utc),
        id,
    }))
}

fn reel_cursor(feed_reel: &reels::FeedReel) -> String {
    format!(
        "{}|{}|{}",
        feed_reel.rank_score,
        feed_reel.reel.created_at.to_rfc3339(),
        feed_reel.reel.id
    )
}

fn map_create_reel_error(error: sqlx::Error) -> AppError {
    match error {
        sqlx::Error::RowNotFound => AppError::BadRequest("media_id"),
        other => AppError::Database(other),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reel_cursor_parser_rejects_non_finite_score() {
        assert!(matches!(
            parse_reel_cursor(Some(
                "inf|2026-01-01T00:00:00Z|00000000-0000-0000-0000-000000000000"
                    .to_owned()
            )),
            Err(AppError::BadRequest("cursor"))
        ));
    }

    #[test]
    fn audio_fields_trim_and_drop_empty_values() {
        assert_eq!(
            normalize_audio_field(Some("  Original sound ".to_owned()), "audio_title")
                .expect("valid"),
            Some("Original sound".to_owned())
        );
        assert_eq!(
            normalize_audio_field(Some("   ".to_owned()), "audio_title").expect("valid"),
            None
        );
    }
}
