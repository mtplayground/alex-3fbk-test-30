use std::collections::HashSet;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, Utc};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use zeroclaw_core::models::UserId;
use zeroclaw_core::repositories::{posts, users};

use crate::error::AppError;
use crate::extractors::{AuthUser, OptionalAuthUser};
use crate::notifications;
use crate::state::AppState;

const DEFAULT_PAGE_LIMIT: i64 = 20;
const MAX_PAGE_LIMIT: i64 = 50;
const MAX_POST_MEDIA: usize = 10;
const FEED_CACHE_TTL_SECONDS: u64 = 30;

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

#[derive(Debug, Deserialize)]
pub struct ExploreQuery {
    cursor: Option<String>,
    limit: Option<i64>,
    hashtag: Option<String>,
    place: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct PostAuthorResponse {
    id: String,
    handle: String,
}

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct PostMentionResponse {
    user_id: String,
    handle: String,
    position: i32,
}

#[derive(Debug, Serialize, Deserialize)]
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
    notifications::emit_mentions(&state, auth_user.id(), post.id, &post.mentions).await;

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
    OptionalAuthUser(auth_user): OptionalAuthUser,
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
    let posts = posts::list_by_author(
        state.db_pool(),
        user.id(),
        auth_user.map(|user| user.id()),
        cursor,
        limit + 1,
    )
    .await?;
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

pub async fn get_feed(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Query(query): Query<PostsQuery>,
) -> Result<Json<PostsPageResponse>, AppError> {
    let limit = query
        .limit
        .unwrap_or(DEFAULT_PAGE_LIMIT)
        .clamp(1, MAX_PAGE_LIMIT);
    let cache_key = feed_cache_key(&state, auth_user.id(), query.cursor.as_deref(), limit);
    let mut redis = state.redis_manager();

    if let Some(cached) = redis.get::<_, Option<String>>(&cache_key).await? {
        if let Ok(response) = serde_json::from_str::<PostsPageResponse>(&cached) {
            return Ok(Json(response));
        }
    }

    let cursor = parse_feed_cursor(query.cursor)?;
    let feed_posts = posts::list_home_feed(state.db_pool(), auth_user.id(), cursor, limit + 1).await?;
    let has_next = feed_posts.len() > limit as usize;
    let page_posts: Vec<_> = feed_posts.into_iter().take(limit as usize).collect();
    let next_cursor = if has_next {
        page_posts.last().map(feed_cursor)
    } else {
        None
    };
    let response = PostsPageResponse {
        posts: page_posts
            .into_iter()
            .map(|feed_post| PostResponse::from(feed_post.post))
            .collect(),
        next_cursor,
    };
    let serialized = serde_json::to_string(&response)
        .map_err(|error| AppError::Internal(format!("feed cache serialization failed: {error}")))?;
    redis
        .set_ex::<_, _, ()>(&cache_key, serialized, FEED_CACHE_TTL_SECONDS)
        .await?;

    Ok(Json(response))
}

pub async fn get_explore(
    State(state): State<AppState>,
    OptionalAuthUser(auth_user): OptionalAuthUser,
    Query(query): Query<ExploreQuery>,
) -> Result<Json<PostsPageResponse>, AppError> {
    let limit = query
        .limit
        .unwrap_or(DEFAULT_PAGE_LIMIT)
        .clamp(1, MAX_PAGE_LIMIT);
    let cursor = parse_feed_cursor(query.cursor)?;
    let hashtag = normalize_hashtag(query.hashtag)?;
    let place = normalize_place(query.place);
    let explore_posts = posts::list_explore(
        state.db_pool(),
        auth_user.map(|user| user.id()),
        hashtag.as_deref(),
        place.as_deref(),
        cursor,
        limit + 1,
    )
    .await?;
    let has_next = explore_posts.len() > limit as usize;
    let page_posts: Vec<_> = explore_posts.into_iter().take(limit as usize).collect();
    let next_cursor = if has_next {
        page_posts.last().map(feed_cursor)
    } else {
        None
    };

    Ok(Json(PostsPageResponse {
        posts: page_posts
            .into_iter()
            .map(|explore_post| PostResponse::from(explore_post.post))
            .collect(),
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

fn normalize_hashtag(hashtag: Option<String>) -> Result<Option<String>, AppError> {
    let Some(hashtag) = hashtag else {
        return Ok(None);
    };
    let hashtag = hashtag.trim().trim_start_matches('#').to_ascii_lowercase();
    if hashtag.is_empty() {
        return Ok(None);
    }
    if hashtag.chars().all(is_entity_char) {
        Ok(Some(hashtag))
    } else {
        Err(AppError::BadRequest("hashtag"))
    }
}

fn normalize_place(place: Option<String>) -> Option<String> {
    place
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

fn parse_feed_cursor(cursor: Option<String>) -> Result<Option<posts::FeedCursor>, AppError> {
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

    Ok(Some(posts::FeedCursor {
        rank_score,
        created_at: created_at.with_timezone(&Utc),
        id,
    }))
}

fn feed_cursor(feed_post: &posts::FeedPost) -> String {
    format!(
        "{}|{}|{}",
        feed_post.rank_score,
        feed_post.post.created_at.to_rfc3339(),
        feed_post.post.id
    )
}

fn feed_cache_key(state: &AppState, user_id: UserId, cursor: Option<&str>, limit: i64) -> String {
    state.redis_namespace().key([
        "feed".to_owned(),
        user_id.to_string(),
        cursor.unwrap_or("first").to_owned(),
        limit.to_string(),
    ])
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

    #[test]
    fn feed_cursor_parser_rejects_bad_rank_score() {
        assert!(matches!(
            parse_feed_cursor(Some(
                "nan|2026-01-01T00:00:00Z|00000000-0000-0000-0000-000000000000"
                    .to_owned()
            )),
            Err(AppError::BadRequest("cursor"))
        ));
    }

    #[test]
    fn normalize_hashtag_strips_marker_and_lowercases() {
        assert_eq!(
            normalize_hashtag(Some(" #Rust_2026 ".to_owned())).expect("valid"),
            Some("rust_2026".to_owned())
        );
    }

    #[test]
    fn normalize_hashtag_rejects_invalid_characters() {
        assert!(matches!(
            normalize_hashtag(Some("bad-tag".to_owned())),
            Err(AppError::BadRequest("hashtag"))
        ));
    }
}
