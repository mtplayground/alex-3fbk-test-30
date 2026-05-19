use std::time::Duration;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Query, State};
use axum::http::header::{AUTHORIZATION, SEC_WEBSOCKET_PROTOCOL};
use axum::http::HeaderMap;
use axum::response::Response;
use chrono::Utc;
use futures_util::{SinkExt, StreamExt};
use redis::AsyncCommands;
use serde::Deserialize;
use serde_json::json;
use tokio::time::{interval, MissedTickBehavior};
use uuid::Uuid;
use zeroclaw_core::auth as core_auth;
use zeroclaw_core::models::{ConversationId, MessageId, UserId};
use zeroclaw_core::redis::RedisChannel;
use zeroclaw_core::repositories::{conversations, users};

use crate::error::AppError;
use crate::state::AppState;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(30);
const PRESENCE_TTL_SECONDS: u64 = 75;
const MAX_CONVERSATION_CHANNELS: usize = 100;

#[derive(Debug, Deserialize)]
pub struct WebSocketQuery {
    token: Option<String>,
    conversations: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ClientEvent {
    Ping,
    Typing {
        conversation_id: Uuid,
        is_typing: Option<bool>,
    },
    Read {
        conversation_id: Uuid,
        message_id: Uuid,
    },
}

pub async fn websocket_handler(
    State(state): State<AppState>,
    Query(query): Query<WebSocketQuery>,
    headers: HeaderMap,
    websocket: WebSocketUpgrade,
) -> Result<Response, AppError> {
    let token = websocket_token(&headers, query.token.as_deref()).ok_or(AppError::Unauthorized)?;
    let user_id = authenticate_websocket(&state, token).await?;
    let conversation_ids = parse_conversation_ids(query.conversations.as_deref())?;
    validate_conversation_subscriptions(&state, user_id, &conversation_ids).await?;
    let channels = subscription_channels(&state, user_id, &conversation_ids);

    Ok(websocket.on_upgrade(move |socket| {
        run_connection(socket, state, user_id, conversation_ids, channels)
    }))
}

async fn authenticate_websocket(state: &AppState, token: &str) -> Result<UserId, AppError> {
    let claims =
        core_auth::verify_access_token(state.jwt(), token).map_err(|_| AppError::Unauthorized)?;
    let user_id = claims.user_id().map_err(|_| AppError::Unauthorized)?;

    if users::find_by_id(state.db_pool(), user_id).await?.is_none() {
        return Err(AppError::Unauthorized);
    }

    Ok(user_id)
}

async fn run_connection(
    socket: WebSocket,
    state: AppState,
    user_id: UserId,
    conversation_ids: Vec<Uuid>,
    channels: Vec<RedisChannel>,
) {
    let channel_names = channels
        .iter()
        .map(|channel| channel.as_str().to_owned())
        .collect::<Vec<_>>();

    let mut pubsub = match state.redis_client().subscribe(&channels).await {
        Ok(pubsub) => pubsub,
        Err(error) => {
            tracing::warn!(%error, %user_id, "failed to subscribe websocket to Redis channels");
            return;
        }
    };

    let (mut sender, mut receiver) = socket.split();
    let ready = json!({
        "type": "ready",
        "user_id": user_id,
        "channels": channel_names,
        "heartbeat_ms": HEARTBEAT_INTERVAL.as_millis(),
    });

    if sender.send(Message::Text(ready.to_string())).await.is_err() {
        return;
    }

    refresh_presence(&state, user_id).await;

    let mut heartbeat = interval(HEARTBEAT_INTERVAL);
    heartbeat.set_missed_tick_behavior(MissedTickBehavior::Delay);
    let mut redis_messages = pubsub.on_message();

    loop {
        tokio::select! {
            _ = heartbeat.tick() => {
                refresh_presence(&state, user_id).await;
                let heartbeat_message = json!({
                    "type": "heartbeat",
                    "interval_ms": HEARTBEAT_INTERVAL.as_millis(),
                });

                if sender.send(Message::Ping(Vec::new())).await.is_err() {
                    break;
                }

                if sender.send(Message::Text(heartbeat_message.to_string())).await.is_err() {
                    break;
                }
            }
            redis_message = redis_messages.next() => {
                let Some(redis_message) = redis_message else {
                    tracing::warn!(%user_id, "Redis pub/sub stream ended for websocket");
                    break;
                };

                match redis_message.get_payload::<String>() {
                    Ok(payload) => {
                        if sender.send(Message::Text(payload)).await.is_err() {
                            break;
                        }
                    }
                    Err(error) => {
                        tracing::warn!(%error, %user_id, "failed to decode Redis pub/sub payload");
                    }
                }
            }
            socket_message = receiver.next() => {
                match socket_message {
                    Some(Ok(Message::Text(text))) => {
                        if handle_client_text(
                            &state,
                            &mut sender,
                            user_id,
                            &conversation_ids,
                            &text,
                        ).await.is_err() {
                            break;
                        }
                    }
                    Some(Ok(Message::Pong(_))) | Some(Ok(Message::Ping(_))) => {}
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(Message::Binary(_))) => {}
                    Some(Err(error)) => {
                        tracing::debug!(%error, %user_id, "websocket receive error");
                        break;
                    }
                }
            }
        }
    }
}

async fn handle_client_text<S>(
    state: &AppState,
    sender: &mut S,
    user_id: UserId,
    conversation_ids: &[Uuid],
    text: &str,
) -> Result<(), axum::Error>
where
    S: futures_util::Sink<Message, Error = axum::Error> + Unpin,
{
    let Ok(event) = serde_json::from_str::<ClientEvent>(text) else {
        return Ok(());
    };

    match event {
        ClientEvent::Ping => {
            sender
                .send(Message::Text(json!({ "type": "pong" }).to_string()))
                .await?;
        }
        ClientEvent::Typing {
            conversation_id,
            is_typing,
        } => {
            if !conversation_ids.contains(&conversation_id) {
                send_client_error(sender, "conversation_not_subscribed").await?;
                return Ok(());
            }

            let payload = json!({
                "type": "typing",
                "conversation_id": conversation_id,
                "user_id": user_id,
                "is_typing": is_typing.unwrap_or(true),
            });
            publish_conversation_event(state, ConversationId::from(conversation_id), payload).await;
        }
        ClientEvent::Read {
            conversation_id,
            message_id,
        } => {
            if !conversation_ids.contains(&conversation_id) {
                send_client_error(sender, "conversation_not_subscribed").await?;
                return Ok(());
            }

            match conversations::update_last_read(
                state.db_pool(),
                ConversationId::from(conversation_id),
                user_id,
                MessageId::from(message_id),
            )
            .await
            {
                Ok(_) => {
                    let payload = json!({
                        "type": "read",
                        "conversation_id": conversation_id,
                        "user_id": user_id,
                        "message_id": message_id,
                    });
                    publish_conversation_event(state, ConversationId::from(conversation_id), payload)
                        .await;
                }
                Err(error) => {
                    tracing::debug!(%error, %user_id, %conversation_id, "failed to update websocket read receipt");
                    send_client_error(sender, "read_failed").await?;
                }
            }
        }
    }

    Ok(())
}

async fn send_client_error<S>(sender: &mut S, code: &'static str) -> Result<(), axum::Error>
where
    S: futures_util::Sink<Message, Error = axum::Error> + Unpin,
{
    sender
        .send(Message::Text(json!({ "type": "error", "code": code }).to_string()))
        .await
}

async fn validate_conversation_subscriptions(
    state: &AppState,
    user_id: UserId,
    conversation_ids: &[Uuid],
) -> Result<(), AppError> {
    for conversation_id in conversation_ids {
        let is_member = conversations::is_member(
            state.db_pool(),
            ConversationId::from(*conversation_id),
            user_id,
        )
        .await?;
        if !is_member {
            return Err(AppError::NotFound);
        }
    }

    Ok(())
}

async fn refresh_presence(state: &AppState, user_id: UserId) {
    let key = state
        .redis_namespace()
        .key(["presence", "user", &user_id.to_string()]);
    let payload = json!({
        "user_id": user_id,
        "status": "online",
        "seen_at": Utc::now(),
    })
    .to_string();
    let mut redis = state.redis_manager();

    if let Err(error) = redis
        .set_ex::<_, _, ()>(&key, payload, PRESENCE_TTL_SECONDS)
        .await
    {
        tracing::warn!(%error, %user_id, "failed to refresh websocket presence");
    }
}

async fn publish_conversation_event(
    state: &AppState,
    conversation_id: ConversationId,
    payload: serde_json::Value,
) {
    let channel = state
        .redis_namespace()
        .channel(["conversation", &conversation_id.to_string()]);
    let mut redis = state.redis_manager();

    match serde_json::to_string(&payload) {
        Ok(serialized) => {
            if let Err(error) = state
                .redis_client()
                .publish(&mut redis, &channel, serialized)
                .await
            {
                tracing::warn!(%error, %conversation_id, "failed to publish websocket event");
            }
        }
        Err(error) => {
            tracing::warn!(%error, %conversation_id, "failed to serialize websocket event");
        }
    }
}

fn subscription_channels(
    state: &AppState,
    user_id: UserId,
    conversation_ids: &[Uuid],
) -> Vec<RedisChannel> {
    let mut channels = Vec::with_capacity(conversation_ids.len() + 1);
    channels.push(
        state
            .redis_namespace()
            .channel(["user", &user_id.to_string()]),
    );

    for conversation_id in conversation_ids {
        channels.push(
            state
                .redis_namespace()
                .channel(["conversation", &conversation_id.to_string()]),
        );
    }

    channels
}

fn websocket_token<'a>(headers: &'a HeaderMap, query_token: Option<&'a str>) -> Option<&'a str> {
    query_token
        .filter(|token| !token.trim().is_empty())
        .map(str::trim)
        .or_else(|| authorization_token(headers))
        .or_else(|| websocket_protocol_token(headers))
}

fn authorization_token(headers: &HeaderMap) -> Option<&str> {
    let header = headers.get(AUTHORIZATION)?.to_str().ok()?;
    parse_bearer_token(header)
}

fn websocket_protocol_token(headers: &HeaderMap) -> Option<&str> {
    let header = headers.get(SEC_WEBSOCKET_PROTOCOL)?.to_str().ok()?;
    let mut saw_bearer = false;

    for part in header.split(',').map(str::trim) {
        if part.eq_ignore_ascii_case("bearer") {
            saw_bearer = true;
            continue;
        }

        if saw_bearer && !part.is_empty() {
            return Some(part);
        }
    }

    None
}

fn parse_bearer_token(header: &str) -> Option<&str> {
    let (scheme, token) = header.trim().split_once(' ')?;
    if !scheme.eq_ignore_ascii_case("Bearer") || token.trim().is_empty() {
        return None;
    }

    Some(token.trim())
}

fn parse_conversation_ids(input: Option<&str>) -> Result<Vec<Uuid>, AppError> {
    let Some(input) = input else {
        return Ok(Vec::new());
    };

    let values = input
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();

    if values.len() > MAX_CONVERSATION_CHANNELS {
        return Err(AppError::BadRequest("too many conversation subscriptions"));
    }

    values
        .into_iter()
        .map(|value| {
            Uuid::parse_str(value).map_err(|_| AppError::BadRequest("invalid conversation id"))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bearer_parser_accepts_authorization_header() {
        assert_eq!(parse_bearer_token("Bearer token"), Some("token"));
        assert_eq!(parse_bearer_token("bearer token"), Some("token"));
    }

    #[test]
    fn bearer_parser_rejects_missing_or_empty_token() {
        assert_eq!(parse_bearer_token("token"), None);
        assert_eq!(parse_bearer_token("Bearer   "), None);
    }

    #[test]
    fn protocol_parser_accepts_bearer_token_pair() {
        let mut headers = HeaderMap::new();
        headers.insert(SEC_WEBSOCKET_PROTOCOL, "bearer, token".parse().unwrap());

        assert_eq!(websocket_protocol_token(&headers), Some("token"));
    }

    #[test]
    fn conversation_parser_allows_empty_input() {
        assert!(parse_conversation_ids(None).unwrap().is_empty());
        assert!(parse_conversation_ids(Some(" , ")).unwrap().is_empty());
    }

    #[test]
    fn conversation_parser_rejects_invalid_uuid() {
        assert!(parse_conversation_ids(Some("not-a-uuid")).is_err());
    }
}
