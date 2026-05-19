use std::time::Duration;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Query, State};
use axum::http::header::{AUTHORIZATION, SEC_WEBSOCKET_PROTOCOL};
use axum::http::HeaderMap;
use axum::response::Response;
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::json;
use tokio::time::{interval, MissedTickBehavior};
use uuid::Uuid;
use zeroclaw_core::auth as core_auth;
use zeroclaw_core::models::UserId;
use zeroclaw_core::redis::RedisChannel;
use zeroclaw_core::repositories::users;

use crate::error::AppError;
use crate::state::AppState;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(30);
const MAX_CONVERSATION_CHANNELS: usize = 100;

#[derive(Debug, Deserialize)]
pub struct WebSocketQuery {
    token: Option<String>,
    conversations: Option<String>,
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
    let channels = subscription_channels(&state, user_id, &conversation_ids);

    Ok(websocket.on_upgrade(move |socket| run_connection(socket, state, user_id, channels)))
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

    let mut heartbeat = interval(HEARTBEAT_INTERVAL);
    heartbeat.set_missed_tick_behavior(MissedTickBehavior::Delay);
    let mut redis_messages = pubsub.on_message();

    loop {
        tokio::select! {
            _ = heartbeat.tick() => {
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
                        if handle_client_text(&mut sender, &text).await.is_err() {
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

async fn handle_client_text<S>(sender: &mut S, text: &str) -> Result<(), axum::Error>
where
    S: futures_util::Sink<Message, Error = axum::Error> + Unpin,
{
    let Ok(value) = serde_json::from_str::<serde_json::Value>(text) else {
        return Ok(());
    };

    if value.get("type").and_then(serde_json::Value::as_str) == Some("ping") {
        sender
            .send(Message::Text(json!({ "type": "pong" }).to_string()))
            .await?;
    }

    Ok(())
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
