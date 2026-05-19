use std::collections::HashMap;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use zeroclaw_core::repositories::comments;

use crate::error::AppError;
use crate::extractors::AuthUser;
use crate::notifications;
use crate::state::AppState;

const MAX_COMMENT_BODY_LENGTH: usize = 2_000;

#[derive(Debug, Deserialize)]
pub struct CreateCommentRequest {
    body: String,
    parent_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct CommentResponse {
    id: String,
    post_id: String,
    parent_id: Option<String>,
    author: CommentAuthorResponse,
    body: String,
    created_at: DateTime<Utc>,
    replies: Vec<CommentResponse>,
}

#[derive(Debug, Serialize)]
pub struct CommentAuthorResponse {
    id: String,
    handle: String,
}

#[derive(Debug, Serialize)]
pub struct CommentsResponse {
    comments: Vec<CommentResponse>,
}

pub async fn create_comment(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(post_id): Path<Uuid>,
    Json(payload): Json<CreateCommentRequest>,
) -> Result<(StatusCode, Json<CommentResponse>), AppError> {
    let parent_id = payload.parent_id;
    let body = validate_body(payload.body)?;
    let input = comments::CreateComment {
        post_id,
        parent_id,
        author_id: auth_user.id(),
        body,
    };
    let comment = comments::create(state.db_pool(), &input)
        .await
        .map_err(|error| map_comment_write_error(error, parent_id))?;
    notifications::emit_comment(&state, auth_user.id(), comment.post_id, comment.id).await;

    Ok((StatusCode::CREATED, Json(CommentResponse::from(comment))))
}

pub async fn get_post_comments(
    State(state): State<AppState>,
    Path(post_id): Path<Uuid>,
) -> Result<Json<CommentsResponse>, AppError> {
    let comments = comments::list_by_post(state.db_pool(), post_id)
        .await
        .map_err(map_comment_read_error)?;

    Ok(Json(CommentsResponse {
        comments: nest_comments(comments),
    }))
}

pub async fn delete_comment(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let deleted = comments::delete(state.db_pool(), id, auth_user.id()).await?;
    if !deleted {
        return Err(AppError::NotFound);
    }

    Ok(StatusCode::NO_CONTENT)
}

impl From<comments::Comment> for CommentResponse {
    fn from(comment: comments::Comment) -> Self {
        Self {
            id: comment.id.to_string(),
            post_id: comment.post_id.to_string(),
            parent_id: comment.parent_id.map(|id| id.to_string()),
            author: CommentAuthorResponse {
                id: comment.author_id.to_string(),
                handle: comment.author_handle,
            },
            body: comment.body,
            created_at: comment.created_at,
            replies: Vec::new(),
        }
    }
}

fn nest_comments(comments: Vec<comments::Comment>) -> Vec<CommentResponse> {
    let mut roots = Vec::new();
    let mut replies: HashMap<Uuid, Vec<CommentResponse>> = HashMap::new();

    for comment in comments {
        if let Some(parent_id) = comment.parent_id {
            replies
                .entry(parent_id)
                .or_default()
                .push(CommentResponse::from(comment));
        } else {
            roots.push(CommentResponse::from(comment));
        }
    }

    for root in &mut roots {
        if let Ok(root_id) = Uuid::parse_str(&root.id) {
            root.replies = replies.remove(&root_id).unwrap_or_default();
        }
    }

    roots
}

fn validate_body(body: String) -> Result<String, AppError> {
    let body = body.trim().to_owned();
    if body.is_empty() {
        return Err(AppError::BadRequest("body"));
    }

    if body.chars().count() > MAX_COMMENT_BODY_LENGTH {
        return Err(AppError::BadRequest("body"));
    }

    Ok(body)
}

fn map_comment_write_error(error: sqlx::Error, parent_id: Option<Uuid>) -> AppError {
    match error {
        sqlx::Error::RowNotFound if parent_id.is_some() => AppError::BadRequest("parent_id"),
        sqlx::Error::RowNotFound => AppError::NotFound,
        other => AppError::Database(other),
    }
}

fn map_comment_read_error(error: sqlx::Error) -> AppError {
    match error {
        sqlx::Error::RowNotFound => AppError::NotFound,
        other => AppError::Database(other),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_comment_body_is_rejected() {
        assert!(matches!(
            validate_body("   ".to_owned()),
            Err(AppError::BadRequest("body"))
        ));
    }
}
