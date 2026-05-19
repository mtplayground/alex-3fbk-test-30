use std::collections::HashSet;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use zeroclaw_core::repositories::{posts, users};

use crate::error::AppError;
use crate::extractors::AuthUser;
use crate::state::AppState;

const DEFAULT_PAGE_LIMIT: i64 = 20;
const MAX_PAGE_LIMIT: i64 = 50;
const MAX_POST_MEDIA: usize = 10;

#[derive(Debug, Deserialize)]
pub struct CreatePostRequest {
    caption: Option<String>,
    location: Option<String>,
    media_ids: Vec<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct PostsQuery {
    cursor: Option<String>,
    limit: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct PostResponse {
    id: String,
    author: PostAuthorResponse,
    caption: String,
    location: Option<String>,
    created_at: DateTime<Utc>,
    media: Vec<PostMediaResponse>,
    hashtags: Vec<String>,
    mentions: Vec<PostMentionResponse>,
}

#[derive(Debug, Serialize)]
pub struct PostAuthorResponse {
    id: String,
    handle: String,
}

#[derive(Debug, Serialize)]
pub struct PostMediaResponse {
    media_id: String,
    position: i32,
    kind: String,
    original_key: String,
    variants: serde_json::Value,
    width: Option<i32>,
    height: Option<i32>,
    duration_ms: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct PostMentionResponse {
    user_id: String,
    handle: String,
    position: i32,
}

#[derive(Debug, Serialize)]
pub struct PostsPageResponse {
    posts: Vec<PostResponse>,
    next_cursor: Option<String>,
}

pub async fn create_post(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<CreatePostRequest>,
) -> Result<(StatusCode, Json<PostResponse>), AppError> {
    validate_media_ids(&payload.media_ids)?;
    let caption = payload.caption.unwrap_or_default().trim().to_owned();
    let location = normalize_location(payload.location);
    let parsed = parse_caption_entities(&caption);
    let input = posts::CreatePost {
        author_id: auth_user.id(),
        caption,
        location,
        media_ids: payload.media_ids,
        hashtags: parsed.hashtags,
        mentions: parsed.mentions,
    };
    let post = posts::create(state.db_pool(), &input)
        .await
        .map_err(map_create_error)?;

    Ok((StatusCode::CREATED, Json(PostResponse::from(post))))
}

pub async fn get_post(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<PostResponse>, AppError> {
    let Some(post) = posts::find_by_id(state.db_pool(), id).await? else {
        return Err(AppError::NotFound);
    };

    Ok(Json(PostResponse::from(post)))
}

pub async fn delete_post(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let deleted = posts::soft_delete(state.db_pool(), id, auth_user.id()).await?;
    if !deleted {
        return Err(AppError::NotFound);
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_user_posts(
    State(state): State<AppState>,
    Path(handle): Path<String>,
    Query(query): Query<PostsQuery>,
) -> Result<Json<PostsPageResponse>, AppError> {
    let Some(user) = users::find_by_handle(state.db_pool(), &handle).await? else {
        return Err(AppError::NotFound);
    };

    let limit = query
        .limit
        .unwrap_or(DEFAULT_PAGE_LIMIT)
        .clamp(1, MAX_PAGE_LIMIT);
    let cursor = parse_cursor(query.cursor)?;
    let posts = posts::list_by_author(state.db_pool(), user.id(), cursor, limit + 1).await?;
    let has_next = posts.len() > limit as usize;
    let page_posts: Vec<_> = posts.into_iter().take(limit as usize).collect();
    let next_cursor = if has_next {
        page_posts.last().map(|post| post.created_at.to_rfc3339())
    } else {
        None
    };

    Ok(Json(PostsPageResponse {
        posts: page_posts.into_iter().map(PostResponse::from).collect(),
        next_cursor,
    }))
}

impl From<posts::Post> for PostResponse {
    fn from(post: posts::Post) -> Self {
        Self {
            id: post.id.to_string(),
            author: PostAuthorResponse {
                id: post.author_id.to_string(),
                handle: post.author_handle,
            },
            caption: post.caption,
            location: post.location,
            created_at: post.created_at,
            media: post
                .media
                .into_iter()
                .map(PostMediaResponse::from)
                .collect(),
            hashtags: post.hashtags,
            mentions: post
                .mentions
                .into_iter()
                .map(PostMentionResponse::from)
                .collect(),
        }
    }
}

impl From<posts::PostMedia> for PostMediaResponse {
    fn from(media: posts::PostMedia) -> Self {
        Self {
            media_id: media.media_id.to_string(),
            position: media.position,
            kind: media.kind,
            original_key: media.original_key,
            variants: media.variants,
            width: media.width,
            height: media.height,
            duration_ms: media.duration_ms,
        }
    }
}

impl From<posts::PostMention> for PostMentionResponse {
    fn from(mention: posts::PostMention) -> Self {
        Self {
            user_id: mention.user_id.to_string(),
            handle: mention.handle,
            position: mention.position,
        }
    }
}

struct ParsedCaptionEntities {
    hashtags: Vec<String>,
    mentions: Vec<posts::ParsedMention>,
}

fn parse_caption_entities(caption: &str) -> ParsedCaptionEntities {
    let mut hashtags = Vec::new();
    let mut seen_hashtags = HashSet::new();
    let mut mentions = Vec::new();
    let bytes = caption.as_bytes();
    let mut index = 0;

    while index < bytes.len() {
        let marker = bytes[index] as char;
        if marker != '#' && marker != '@' {
            index += 1;
            continue;
        }

        if index > 0 && is_entity_char(bytes[index - 1] as char) {
            index += 1;
            continue;
        }

        let start = index + 1;
        let mut end = start;
        while end < bytes.len() && is_entity_char(bytes[end] as char) {
            end += 1;
        }

        if end == start {
            index += 1;
            continue;
        }

        let value = caption[start..end].to_ascii_lowercase();
        if marker == '#' {
            if seen_hashtags.insert(value.clone()) {
                hashtags.push(value);
            }
        } else {
            mentions.push(posts::ParsedMention {
                handle: value,
                position: index as i32,
            });
        }

        index = end;
    }

    ParsedCaptionEntities { hashtags, mentions }
}

fn is_entity_char(value: char) -> bool {
    value.is_ascii_alphanumeric() || value == '_'
}

fn validate_media_ids(media_ids: &[Uuid]) -> Result<(), AppError> {
    if media_ids.is_empty() {
        return Err(AppError::BadRequest("media_ids"));
    }

    if media_ids.len() > MAX_POST_MEDIA {
        return Err(AppError::BadRequest("media_ids"));
    }

    let unique_count = media_ids.iter().copied().collect::<HashSet<_>>().len();
    if unique_count != media_ids.len() {
        return Err(AppError::BadRequest("media_ids"));
    }

    Ok(())
}

fn normalize_location(location: Option<String>) -> Option<String> {
    location
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

fn parse_cursor(cursor: Option<String>) -> Result<Option<DateTime<Utc>>, AppError> {
    let Some(cursor) = cursor else {
        return Ok(None);
    };
    let cursor = cursor.trim();
    if cursor.is_empty() {
        return Ok(None);
    }

    let parsed =
        DateTime::parse_from_rfc3339(cursor).map_err(|_| AppError::BadRequest("cursor"))?;
    Ok(Some(parsed.with_timezone(&Utc)))
}

fn map_create_error(error: sqlx::Error) -> AppError {
    match error {
        sqlx::Error::RowNotFound => AppError::BadRequest("media_ids"),
        other => AppError::Database(other),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn caption_parser_extracts_unique_hashtags_and_mentions() {
        let parsed = parse_caption_entities("Hi @Mira #Rust #rust email@test @noor_2");

        assert_eq!(parsed.hashtags, vec!["rust"]);
        assert_eq!(parsed.mentions.len(), 2);
        assert_eq!(parsed.mentions[0].handle, "mira");
        assert_eq!(parsed.mentions[1].handle, "noor_2");
    }

    #[test]
    fn cursor_parser_rejects_invalid_dates() {
        assert!(matches!(
            parse_cursor(Some("not-a-date".to_owned())),
            Err(AppError::BadRequest("cursor"))
        ));
    }
}
