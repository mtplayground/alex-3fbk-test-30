use axum::extract::{Query, State};
use axum::Json;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use zeroclaw_core::models::{ConversationId, UserId};
use zeroclaw_core::repositories::notifications::{
    self, CreateNotification, NotificationKind, NotificationTargetKind,
};
use zeroclaw_core::repositories::{conversations, posts, social::LikeTargetKind};

use crate::error::AppError;
use crate::extractors::AuthUser;
use crate::state::AppState;

const DEFAULT_PAGE_LIMIT: i64 = 20;
const MAX_PAGE_LIMIT: i64 = 50;

#[derive(Debug, Deserialize)]
pub struct NotificationsQuery {
    cursor: Option<String>,
    limit: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct NotificationResponse {
    id: String,
    user_id: String,
    kind: &'static str,
    actor_id: String,
    target_kind: &'static str,
    target_id: String,
    read_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct NotificationsPageResponse {
    notifications: Vec<NotificationResponse>,
    next_cursor: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UnreadCountResponse {
    unread_count: i64,
}

#[derive(Debug, Serialize)]
pub struct ReadAllResponse {
    updated_count: u64,
}

pub async fn list_notifications(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Query(query): Query<NotificationsQuery>,
) -> Result<Json<NotificationsPageResponse>, AppError> {
    let limit = query
        .limit
        .unwrap_or(DEFAULT_PAGE_LIMIT)
        .clamp(1, MAX_PAGE_LIMIT);
    let cursor = parse_notification_cursor(query.cursor)?;
    let notifications =
        notifications::list_for_user(state.db_pool(), auth_user.id(), cursor, limit + 1).await?;
    let has_next = notifications.len() > limit as usize;
    let page_notifications = notifications
        .into_iter()
        .take(limit as usize)
        .collect::<Vec<_>>();
    let next_cursor = if has_next {
        page_notifications.last().map(notification_cursor)
    } else {
        None
    };

    Ok(Json(NotificationsPageResponse {
        notifications: page_notifications
            .into_iter()
            .map(NotificationResponse::from)
            .collect(),
        next_cursor,
    }))
}

pub async fn mark_all_notifications_read(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<Json<ReadAllResponse>, AppError> {
    let updated_count = notifications::mark_all_read(state.db_pool(), auth_user.id()).await?;

    Ok(Json(ReadAllResponse { updated_count }))
}

pub async fn unread_notification_count(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<Json<UnreadCountResponse>, AppError> {
    let unread_count = notifications::unread_count(state.db_pool(), auth_user.id()).await?;

    Ok(Json(UnreadCountResponse { unread_count }))
}

pub async fn emit_like(
    state: &AppState,
    actor_id: UserId,
    target_kind: LikeTargetKind,
    target_id: Uuid,
) {
    let recipient = match like_recipient(state, target_kind, target_id).await {
        Ok(Some(recipient)) => recipient,
        Ok(None) => return,
        Err(error) => {
            tracing::warn!(%error, %target_id, "failed to resolve like notification recipient");
            return;
        }
    };

    emit(
        state,
        CreateNotification {
            user_id: recipient,
            kind: NotificationKind::Like,
            actor_id,
            target_kind: match target_kind {
                LikeTargetKind::Post => NotificationTargetKind::Post,
                LikeTargetKind::Comment => NotificationTargetKind::Comment,
            },
            target_id,
        },
    )
    .await;
}

pub async fn emit_comment(
    state: &AppState,
    actor_id: UserId,
    post_id: Uuid,
    comment_id: Uuid,
) {
    let recipient = match post_author_id(state, post_id).await {
        Ok(Some(recipient)) => recipient,
        Ok(None) => return,
        Err(error) => {
            tracing::warn!(%error, %post_id, "failed to resolve comment notification recipient");
            return;
        }
    };

    emit(
        state,
        CreateNotification {
            user_id: recipient,
            kind: NotificationKind::Comment,
            actor_id,
            target_kind: NotificationTargetKind::Comment,
            target_id: comment_id,
        },
    )
    .await;
}

pub async fn emit_follow(state: &AppState, actor_id: UserId, followee_id: UserId) {
    emit(
        state,
        CreateNotification {
            user_id: followee_id,
            kind: NotificationKind::Follow,
            actor_id,
            target_kind: NotificationTargetKind::User,
            target_id: actor_id.as_uuid(),
        },
    )
    .await;
}

pub async fn emit_mentions(
    state: &AppState,
    actor_id: UserId,
    post_id: Uuid,
    mentions: &[posts::PostMention],
) {
    for mention in mentions {
        emit(
            state,
            CreateNotification {
                user_id: mention.user_id,
                kind: NotificationKind::Mention,
                actor_id,
                target_kind: NotificationTargetKind::Post,
                target_id: post_id,
            },
        )
        .await;
    }
}

pub async fn emit_dm(
    state: &AppState,
    actor_id: UserId,
    conversation_id: Uuid,
    message_id: Uuid,
) {
    let members = match conversations::list_members(
        state.db_pool(),
        ConversationId::from(conversation_id),
    )
    .await
    {
        Ok(members) => members,
        Err(error) => {
            tracing::warn!(%error, %conversation_id, "failed to resolve dm notification recipients");
            return;
        }
    };

    for member in members {
        emit(
            state,
            CreateNotification {
                user_id: member.user_id(),
                kind: NotificationKind::Dm,
                actor_id,
                target_kind: NotificationTargetKind::Message,
                target_id: message_id,
            },
        )
        .await;
    }
}

async fn emit(state: &AppState, input: CreateNotification) {
    let notification = match notifications::create(state.db_pool(), &input).await {
        Ok(Some(notification)) => notification,
        Ok(None) => return,
        Err(error) => {
            tracing::warn!(
                %error,
                user_id = %input.user_id,
                actor_id = %input.actor_id,
                "failed to create notification"
            );
            return;
        }
    };

    publish(state, NotificationResponse::from(notification)).await;
}

async fn publish(state: &AppState, event: NotificationResponse) {
    let channel = state.redis_namespace().channel(["user", &event.user_id]);
    let payload = match serde_json::to_string(&serde_json::json!({
        "type": "notification",
        "notification": event,
    })) {
        Ok(payload) => payload,
        Err(error) => {
            tracing::warn!(%error, "failed to serialize notification event");
            return;
        }
    };
    let mut redis = state.redis_manager();

    if let Err(error) = state
        .redis_client()
        .publish(&mut redis, &channel, payload)
        .await
    {
        tracing::warn!(%error, "failed to publish notification event");
    }
}

async fn like_recipient(
    state: &AppState,
    target_kind: LikeTargetKind,
    target_id: Uuid,
) -> sqlx::Result<Option<UserId>> {
    match target_kind {
        LikeTargetKind::Post => post_author_id(state, target_id).await,
        LikeTargetKind::Comment => comment_author_id(state, target_id).await,
    }
}

async fn post_author_id(state: &AppState, post_id: Uuid) -> sqlx::Result<Option<UserId>> {
    let row: Option<Uuid> = sqlx::query_scalar(
        r#"
        SELECT author_id
        FROM posts
        WHERE id = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(post_id)
    .fetch_optional(state.db_pool())
    .await?;

    Ok(row.map(UserId::from))
}

async fn comment_author_id(state: &AppState, comment_id: Uuid) -> sqlx::Result<Option<UserId>> {
    let row: Option<Uuid> = sqlx::query_scalar(
        r#"
        SELECT comments.author_id
        FROM comments
        JOIN posts ON posts.id = comments.post_id
        WHERE comments.id = $1 AND posts.deleted_at IS NULL
        "#,
    )
    .bind(comment_id)
    .fetch_optional(state.db_pool())
    .await?;

    Ok(row.map(UserId::from))
}

fn parse_notification_cursor(
    cursor: Option<String>,
) -> Result<Option<notifications::NotificationCursor>, AppError> {
    let Some(cursor) = cursor else {
        return Ok(None);
    };
    let cursor = cursor.trim();
    if cursor.is_empty() {
        return Ok(None);
    }

    let mut parts = cursor.split('|');
    let Some(created_at) = parts.next() else {
        return Err(AppError::BadRequest("cursor"));
    };
    let Some(id) = parts.next() else {
        return Err(AppError::BadRequest("cursor"));
    };
    if parts.next().is_some() {
        return Err(AppError::BadRequest("cursor"));
    }

    let created_at =
        DateTime::parse_from_rfc3339(created_at).map_err(|_| AppError::BadRequest("cursor"))?;
    let id = Uuid::parse_str(id).map_err(|_| AppError::BadRequest("cursor"))?;

    Ok(Some(notifications::NotificationCursor {
        created_at: created_at.with_timezone(&Utc),
        id,
    }))
}

fn notification_cursor(notification: &notifications::Notification) -> String {
    format!("{}|{}", notification.created_at.to_rfc3339(), notification.id)
}

impl From<notifications::Notification> for NotificationResponse {
    fn from(notification: notifications::Notification) -> Self {
        Self {
            id: notification.id.to_string(),
            user_id: notification.user_id.to_string(),
            kind: notification.kind.as_str(),
            actor_id: notification.actor_id.to_string(),
            target_kind: notification.target_kind.as_str(),
            target_id: notification.target_id.to_string(),
            read_at: notification.read_at,
            created_at: notification.created_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notification_response_maps_storage_values() {
        assert_eq!(NotificationKind::Like.as_str(), "like");
        assert_eq!(NotificationKind::Dm.as_str(), "dm");
        assert_eq!(NotificationTargetKind::Message.as_str(), "message");
    }
}
