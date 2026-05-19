use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use zeroclaw_core::repositories::stories;

use crate::error::AppError;
use crate::extractors::AuthUser;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct CreateStoryRequest {
    media_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct StoryResponse {
    id: String,
    author: StoryAuthorResponse,
    media: StoryMediaResponse,
    created_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
    viewer_count: i64,
    viewed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct StoryAuthorResponse {
    id: String,
    handle: String,
    display_name: String,
    avatar_key: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct StoryMediaResponse {
    media_id: String,
    kind: String,
    status: String,
    original_key: String,
    variants: serde_json::Value,
    width: Option<i32>,
    height: Option<i32>,
    duration_ms: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct StoriesFeedResponse {
    authors: Vec<StoryAuthorStoriesResponse>,
}

#[derive(Debug, Serialize)]
pub struct StoryAuthorStoriesResponse {
    author: StoryAuthorResponse,
    stories: Vec<StoryResponse>,
}

#[derive(Debug, Serialize)]
pub struct StoryViewersResponse {
    viewers: Vec<StoryViewerResponse>,
}

#[derive(Debug, Serialize)]
pub struct StoryViewerResponse {
    id: String,
    handle: String,
    display_name: String,
    avatar_key: Option<String>,
    viewed_at: DateTime<Utc>,
}

pub async fn create_story(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<CreateStoryRequest>,
) -> Result<(StatusCode, Json<StoryResponse>), AppError> {
    let input = stories::CreateStory {
        author_id: auth_user.id(),
        media_id: payload.media_id,
    };
    let story = stories::create(state.db_pool(), &input)
        .await
        .map_err(map_create_story_error)?;

    Ok((StatusCode::CREATED, Json(StoryResponse::from(story))))
}

pub async fn get_stories_feed(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<Json<StoriesFeedResponse>, AppError> {
    let stories = stories::list_feed(state.db_pool(), auth_user.id()).await?;

    Ok(Json(StoriesFeedResponse {
        authors: group_stories_by_author(stories),
    }))
}

pub async fn view_story(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    stories::mark_viewed(state.db_pool(), id, auth_user.id())
        .await
        .map_err(map_story_not_found)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_story_viewers(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<StoryViewersResponse>, AppError> {
    let viewers = stories::list_viewers(state.db_pool(), id, auth_user.id())
        .await
        .map_err(map_story_not_found)?;

    Ok(Json(StoryViewersResponse {
        viewers: viewers.into_iter().map(StoryViewerResponse::from).collect(),
    }))
}

impl From<stories::Story> for StoryResponse {
    fn from(story: stories::Story) -> Self {
        Self {
            id: story.id.to_string(),
            author: StoryAuthorResponse::from(story.author),
            media: StoryMediaResponse::from(story.media),
            created_at: story.created_at,
            expires_at: story.expires_at,
            viewer_count: story.viewer_count,
            viewed_at: story.viewed_at,
        }
    }
}

impl From<stories::StoryAuthor> for StoryAuthorResponse {
    fn from(author: stories::StoryAuthor) -> Self {
        Self {
            id: author.id.to_string(),
            handle: author.handle,
            display_name: author.display_name,
            avatar_key: author.avatar_key,
        }
    }
}

impl From<stories::StoryMedia> for StoryMediaResponse {
    fn from(media: stories::StoryMedia) -> Self {
        Self {
            media_id: media.id.to_string(),
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

impl From<stories::StoryViewer> for StoryViewerResponse {
    fn from(viewer: stories::StoryViewer) -> Self {
        Self {
            id: viewer.id.to_string(),
            handle: viewer.handle,
            display_name: viewer.display_name,
            avatar_key: viewer.avatar_key,
            viewed_at: viewer.viewed_at,
        }
    }
}

fn group_stories_by_author(stories: Vec<stories::Story>) -> Vec<StoryAuthorStoriesResponse> {
    let mut index_by_author = HashMap::new();
    let mut groups: Vec<StoryAuthorStoriesResponse> = Vec::new();

    for story in stories {
        let author_id = story.author.id;
        let response = StoryResponse::from(story);
        let index = *index_by_author.entry(author_id).or_insert_with(|| {
            groups.push(StoryAuthorStoriesResponse {
                author: StoryAuthorResponse {
                    id: response.author.id.clone(),
                    handle: response.author.handle.clone(),
                    display_name: response.author.display_name.clone(),
                    avatar_key: response.author.avatar_key.clone(),
                },
                stories: Vec::new(),
            });
            groups.len() - 1
        });

        groups[index].stories.push(response);
    }

    groups
}

fn map_create_story_error(error: sqlx::Error) -> AppError {
    match error {
        sqlx::Error::RowNotFound => AppError::BadRequest("media_id"),
        other => AppError::Database(other),
    }
}

fn map_story_not_found(error: sqlx::Error) -> AppError {
    match error {
        sqlx::Error::RowNotFound => AppError::NotFound,
        other => AppError::Database(other),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zeroclaw_core::models::UserId;

    #[test]
    fn feed_grouping_keeps_author_story_order() {
        let author_id = UserId::from(Uuid::nil());
        let stories = vec![
            story_with_author(author_id, "first"),
            story_with_author(author_id, "second"),
        ];
        let grouped = group_stories_by_author(stories);

        assert_eq!(grouped.len(), 1);
        assert_eq!(grouped[0].stories.len(), 2);
        assert_eq!(grouped[0].author.handle, "first");
    }

    fn story_with_author(author_id: UserId, handle: &str) -> stories::Story {
        stories::Story {
            id: Uuid::new_v4(),
            author: stories::StoryAuthor {
                id: author_id,
                handle: handle.to_owned(),
                display_name: handle.to_owned(),
                avatar_key: None,
            },
            media: stories::StoryMedia {
                id: Uuid::new_v4(),
                kind: "image".to_owned(),
                status: "ready".to_owned(),
                original_key: "media/original.jpg".to_owned(),
                variants: serde_json::json!({}),
                width: None,
                height: None,
                duration_ms: None,
            },
            created_at: Utc::now(),
            expires_at: Utc::now(),
            viewer_count: 0,
            viewed_at: None,
        }
    }
}
