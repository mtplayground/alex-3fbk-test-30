use std::collections::BTreeSet;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;
use zeroclaw_core::models::{
    Conversation, ConversationId, ConversationKind, ConversationMember, CreateConversation,
    CreateMessage, MediaAssetId, Message, MessageId, UserId,
};
use zeroclaw_core::repositories::conversations;

use crate::error::AppError;
use crate::extractors::AuthUser;
use crate::notifications;
use crate::state::AppState;

const DEFAULT_PAGE_LIMIT: i64 = 20;
const MAX_PAGE_LIMIT: i64 = 50;
const MAX_MESSAGE_BODY_LENGTH: usize = 4_000;
const MAX_CONVERSATION_MEMBERS: usize = 50;

#[derive(Debug, Deserialize)]
pub struct CreateConversationRequest {
    kind: String,
    title: Option<String>,
    member_ids: Vec<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct MessagePageQuery {
    cursor: Option<String>,
    limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct CreateMessageRequest {
    body: Option<String>,
    media_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct MarkReadRequest {
    message_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct ConversationResponse {
    id: String,
    kind: String,
    title: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    members: Vec<ConversationMemberResponse>,
}

#[derive(Debug, Serialize)]
pub struct ConversationMemberResponse {
    user_id: String,
    joined_at: DateTime<Utc>,
    last_read_message_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ConversationsResponse {
    conversations: Vec<ConversationResponse>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MessageResponse {
    id: String,
    conversation_id: String,
    author_id: String,
    body: String,
    media_id: Option<String>,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct MessagesPageResponse {
    messages: Vec<MessageResponse>,
    next_cursor: Option<String>,
}

pub async fn list_conversations(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<Json<ConversationsResponse>, AppError> {
    let conversations =
        conversations::list_for_user(state.db_pool(), auth_user.id(), MAX_PAGE_LIMIT).await?;
    let mut responses = Vec::with_capacity(conversations.len());

    for conversation in conversations {
        responses.push(conversation_response(state.db_pool(), conversation).await?);
    }

    Ok(Json(ConversationsResponse {
        conversations: responses,
    }))
}

pub async fn create_conversation(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<CreateConversationRequest>,
) -> Result<(StatusCode, Json<ConversationResponse>), AppError> {
    let kind = parse_conversation_kind(&payload.kind)?;
    let title = normalize_title(payload.title);
    let members = normalize_members(auth_user.id(), payload.member_ids)?;
    validate_member_count(kind, members.len())?;
    ensure_members_exist(state.db_pool(), &members).await?;
    let mut input = CreateConversation::new(kind);

    if let Some(title) = title {
        input = input.with_title(title);
    }

    let conversation = conversations::create(state.db_pool(), &input).await?;
    for member_id in members {
        conversations::add_member(state.db_pool(), conversation.id(), member_id)
            .await
            .map_err(map_member_write_error)?;
    }

    Ok((
        StatusCode::CREATED,
        Json(conversation_response(state.db_pool(), conversation).await?),
    ))
}

pub async fn list_messages(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Query(query): Query<MessagePageQuery>,
) -> Result<Json<MessagesPageResponse>, AppError> {
    let conversation_id = ConversationId::from(id);
    ensure_member(&state, conversation_id, auth_user.id()).await?;
    let limit = query
        .limit
        .unwrap_or(DEFAULT_PAGE_LIMIT)
        .clamp(1, MAX_PAGE_LIMIT);
    let cursor = parse_message_cursor(query.cursor)?;
    let messages =
        conversations::list_messages(state.db_pool(), conversation_id, cursor, limit + 1).await?;
    let has_next = messages.len() > limit as usize;
    let page_messages = messages.into_iter().take(limit as usize).collect::<Vec<_>>();
    let next_cursor = if has_next {
        page_messages
            .last()
            .map(|message| message.created_at().to_rfc3339())
    } else {
        None
    };

    Ok(Json(MessagesPageResponse {
        messages: page_messages.into_iter().map(MessageResponse::from).collect(),
        next_cursor,
    }))
}

pub async fn create_message(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<CreateMessageRequest>,
) -> Result<(StatusCode, Json<MessageResponse>), AppError> {
    let conversation_id = ConversationId::from(id);
    let body = validate_message_body(payload.body)?;
    let media_id = payload.media_id.map(MediaAssetId::from);

    if body.is_empty() && media_id.is_none() {
        return Err(AppError::BadRequest("body_or_media_id"));
    }

    let mut input = CreateMessage::new(conversation_id, auth_user.id(), body);
    if let Some(media_id) = media_id {
        input = input.with_media_id(media_id);
    }

    let message = conversations::create_message(state.db_pool(), &input)
        .await
        .map_err(map_message_write_error)?;
    let message_id = message.id().as_uuid();
    let response = MessageResponse::from(message);
    publish_message_event(&state, conversation_id, &response).await;
    notifications::emit_dm(
        &state,
        auth_user.id(),
        conversation_id.as_uuid(),
        message_id,
    )
    .await;

    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn mark_conversation_read(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<MarkReadRequest>,
) -> Result<Json<ConversationMemberResponse>, AppError> {
    let member = conversations::update_last_read(
        state.db_pool(),
        ConversationId::from(id),
        auth_user.id(),
        MessageId::from(payload.message_id),
    )
    .await
    .map_err(map_mark_read_error)?;

    Ok(Json(ConversationMemberResponse::from(member)))
}

impl From<ConversationMember> for ConversationMemberResponse {
    fn from(member: ConversationMember) -> Self {
        Self {
            user_id: member.user_id().to_string(),
            joined_at: *member.joined_at(),
            last_read_message_id: member.last_read_message_id().map(|id| id.to_string()),
        }
    }
}

impl From<Message> for MessageResponse {
    fn from(message: Message) -> Self {
        Self {
            id: message.id().to_string(),
            conversation_id: message.conversation_id().to_string(),
            author_id: message.author_id().to_string(),
            body: message.body().to_owned(),
            media_id: message.media_id().map(|id| id.to_string()),
            created_at: *message.created_at(),
        }
    }
}

async fn conversation_response(
    pool: &sqlx::PgPool,
    conversation: Conversation,
) -> Result<ConversationResponse, AppError> {
    let members = conversations::list_members(pool, conversation.id()).await?;

    Ok(ConversationResponse {
        id: conversation.id().to_string(),
        kind: conversation.kind().as_str().to_owned(),
        title: conversation.title().map(str::to_owned),
        created_at: *conversation.created_at(),
        updated_at: *conversation.updated_at(),
        members: members
            .into_iter()
            .map(ConversationMemberResponse::from)
            .collect(),
    })
}

async fn ensure_member(
    state: &AppState,
    conversation_id: ConversationId,
    user_id: UserId,
) -> Result<(), AppError> {
    if conversations::is_member(state.db_pool(), conversation_id, user_id).await? {
        return Ok(());
    }

    Err(AppError::NotFound)
}

fn parse_conversation_kind(value: &str) -> Result<ConversationKind, AppError> {
    ConversationKind::from_str(&value.trim().to_ascii_lowercase())
        .ok_or(AppError::BadRequest("kind"))
}

fn normalize_title(title: Option<String>) -> Option<String> {
    title
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

fn normalize_members(auth_user_id: UserId, member_ids: Vec<Uuid>) -> Result<Vec<UserId>, AppError> {
    let mut ids = BTreeSet::new();
    ids.insert(auth_user_id.as_uuid());

    for member_id in member_ids {
        ids.insert(member_id);
    }

    if ids.len() < 2 || ids.len() > MAX_CONVERSATION_MEMBERS {
        return Err(AppError::BadRequest("member_ids"));
    }

    Ok(ids.into_iter().map(UserId::from).collect())
}

fn validate_member_count(kind: ConversationKind, member_count: usize) -> Result<(), AppError> {
    match kind {
        ConversationKind::Dm if member_count == 2 => Ok(()),
        ConversationKind::Group if (2..=MAX_CONVERSATION_MEMBERS).contains(&member_count) => Ok(()),
        _ => Err(AppError::BadRequest("member_ids")),
    }
}

async fn ensure_members_exist(pool: &sqlx::PgPool, members: &[UserId]) -> Result<(), AppError> {
    let member_ids = members
        .iter()
        .map(|member_id| member_id.as_uuid())
        .collect::<Vec<_>>();
    let count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::bigint
        FROM users
        WHERE id = ANY($1)
        "#,
    )
    .bind(&member_ids)
    .fetch_one(pool)
    .await?;

    if count == member_ids.len() as i64 {
        return Ok(());
    }

    Err(AppError::BadRequest("member_ids"))
}

fn validate_message_body(body: Option<String>) -> Result<String, AppError> {
    let body = body.unwrap_or_default().trim().to_owned();

    if body.chars().count() > MAX_MESSAGE_BODY_LENGTH {
        return Err(AppError::BadRequest("body"));
    }

    Ok(body)
}

fn parse_message_cursor(cursor: Option<String>) -> Result<Option<DateTime<Utc>>, AppError> {
    cursor
        .map(|value| {
            DateTime::parse_from_rfc3339(value.trim())
                .map(|parsed| parsed.with_timezone(&Utc))
                .map_err(|_| AppError::BadRequest("cursor"))
        })
        .transpose()
}

async fn publish_message_event(
    state: &AppState,
    conversation_id: ConversationId,
    message: &MessageResponse,
) {
    let channel = state
        .redis_namespace()
        .channel(["conversation", &conversation_id.to_string()]);
    let payload = match serde_json::to_string(&json!({
        "type": "message",
        "conversation_id": conversation_id,
        "message": message,
    })) {
        Ok(payload) => payload,
        Err(error) => {
            tracing::warn!(%error, %conversation_id, "failed to serialize message event");
            return;
        }
    };
    let mut redis = state.redis_manager();

    if let Err(error) = state
        .redis_client()
        .publish(&mut redis, &channel, payload)
        .await
    {
        tracing::warn!(%error, %conversation_id, "failed to publish message event");
    }
}

fn map_member_write_error(error: sqlx::Error) -> AppError {
    match error {
        sqlx::Error::Database(database_error) => {
            if database_error.constraint() == Some("conversation_members_user_id_fkey") {
                AppError::BadRequest("member_ids")
            } else {
                AppError::Database(sqlx::Error::Database(database_error))
            }
        }
        other => AppError::Database(other),
    }
}

fn map_message_write_error(error: sqlx::Error) -> AppError {
    match error {
        sqlx::Error::RowNotFound => AppError::NotFound,
        sqlx::Error::Database(database_error) => match database_error.constraint() {
            Some("messages_body_or_media_present") => AppError::BadRequest("body_or_media_id"),
            Some("messages_media_id_fkey") => AppError::BadRequest("media_id"),
            _ => AppError::Database(sqlx::Error::Database(database_error)),
        },
        other => AppError::Database(other),
    }
}

fn map_mark_read_error(error: sqlx::Error) -> AppError {
    match error {
        sqlx::Error::RowNotFound => AppError::NotFound,
        other => AppError::Database(other),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_body_validation_trims_and_limits() {
        assert_eq!(
            validate_message_body(Some("  hello  ".to_owned())).expect("body should validate"),
            "hello"
        );
        assert!(matches!(
            validate_message_body(Some("x".repeat(MAX_MESSAGE_BODY_LENGTH + 1))),
            Err(AppError::BadRequest("body"))
        ));
    }

    #[test]
    fn member_normalization_includes_authenticated_user_and_dedupes() {
        let auth_user_id = UserId::from(Uuid::new_v4());
        let other_id = Uuid::new_v4();
        let members = normalize_members(auth_user_id, vec![other_id, other_id])
            .expect("members should validate");

        assert_eq!(members.len(), 2);
        assert!(members.contains(&auth_user_id));
        assert!(members.contains(&UserId::from(other_id)));
    }

    #[test]
    fn member_normalization_requires_at_least_two_members() {
        let auth_user_id = UserId::from(Uuid::new_v4());

        assert!(matches!(
            normalize_members(auth_user_id, Vec::new()),
            Err(AppError::BadRequest("member_ids"))
        ));
    }

    #[test]
    fn dm_conversations_require_exactly_two_members() {
        assert!(validate_member_count(ConversationKind::Dm, 2).is_ok());
        assert!(matches!(
            validate_member_count(ConversationKind::Dm, 3),
            Err(AppError::BadRequest("member_ids"))
        ));
    }
}
