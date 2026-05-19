use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::Body;
use axum::extract::{MatchedPath, State};
use axum::http::{HeaderMap, Method, Request};
use axum::middleware::Next;
use axum::response::Response;
use zeroclaw_core::auth;

use crate::error::AppError;
use crate::state::AppState;

const TOKEN_BUCKET_LUA: &str = r#"
local key = KEYS[1]
local capacity = tonumber(ARGV[1])
local refill_per_second = tonumber(ARGV[2])
local cost = tonumber(ARGV[3])
local now_ms = tonumber(ARGV[4])
local ttl_ms = tonumber(ARGV[5])

local bucket = redis.call('HMGET', key, 'tokens', 'updated_at')
local tokens = tonumber(bucket[1])
local updated_at = tonumber(bucket[2])

if tokens == nil then
  tokens = capacity
end

if updated_at == nil then
  updated_at = now_ms
end

local elapsed_ms = math.max(0, now_ms - updated_at)
tokens = math.min(capacity, tokens + (elapsed_ms * refill_per_second / 1000))

local allowed = 0
local retry_ms = 0
if tokens >= cost then
  tokens = tokens - cost
  allowed = 1
else
  retry_ms = math.ceil((cost - tokens) * 1000 / refill_per_second)
end

redis.call('HMSET', key, 'tokens', tokens, 'updated_at', now_ms)
redis.call('PEXPIRE', key, ttl_ms)

return { allowed, math.floor(tokens), retry_ms }
"#;

#[derive(Debug, Clone, Copy)]
pub struct RateLimitPolicy {
    capacity: u32,
    refill_per_second: f64,
    ttl_ms: u64,
}

impl RateLimitPolicy {
    pub const fn new(capacity: u32, refill_per_second: f64, ttl_ms: u64) -> Self {
        Self {
            capacity,
            refill_per_second,
            ttl_ms,
        }
    }
}

const WRITE_POLICY: RateLimitPolicy = RateLimitPolicy::new(60, 1.0, 120_000);
const LOGIN_POLICY: RateLimitPolicy = RateLimitPolicy::new(5, 1.0 / 180.0, 1_800_000);

pub async fn rate_limit_write_requests(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    if is_write_method(request.method()) {
        let key = write_limit_key(&state, &request);
        check_token_bucket(&state, key, WRITE_POLICY).await?;
    }

    Ok(next.run(request).await)
}

pub async fn check_login_throttle(
    state: &AppState,
    headers: &HeaderMap,
    email: &str,
) -> Result<(), AppError> {
    let ip = client_ip(headers);
    let normalized_email = email.trim().to_ascii_lowercase();
    let email_hash = auth::hash_opaque_token(&normalized_email);
    let key = state
        .redis_namespace()
        .key(["rate", "login", &key_part(&ip), &email_hash]);

    check_token_bucket(state, key, LOGIN_POLICY).await
}

async fn check_token_bucket(
    state: &AppState,
    key: String,
    policy: RateLimitPolicy,
) -> Result<(), AppError> {
    let mut redis = state.redis_manager();
    let now_ms = unix_time_ms()?;
    let result: Vec<i64> = redis::cmd("EVAL")
        .arg(TOKEN_BUCKET_LUA)
        .arg(1)
        .arg(key)
        .arg(policy.capacity)
        .arg(policy.refill_per_second)
        .arg(1_u32)
        .arg(now_ms)
        .arg(policy.ttl_ms)
        .query_async(&mut redis)
        .await?;

    match result.first() {
        Some(1) => Ok(()),
        Some(_) => Err(AppError::RateLimited),
        None => Err(AppError::Internal("rate limit script returned no result".to_owned())),
    }
}

fn write_limit_key(state: &AppState, request: &Request<Body>) -> String {
    let route = request
        .extensions()
        .get::<MatchedPath>()
        .map(MatchedPath::as_str)
        .unwrap_or_else(|| request.uri().path());
    let ip = client_ip(request.headers());

    state
        .redis_namespace()
        .key([
            "rate",
            "write",
            &key_part(request.method().as_str()),
            &key_part(route),
            &key_part(&ip),
        ])
}

fn is_write_method(method: &Method) -> bool {
    matches!(
        method,
        &Method::POST | &Method::PUT | &Method::PATCH | &Method::DELETE
    )
}

fn client_ip(headers: &HeaderMap) -> String {
    headers
        .get("fly-client-ip")
        .or_else(|| headers.get("x-real-ip"))
        .or_else(|| headers.get("x-forwarded-for"))
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(',').next())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("unknown")
        .to_owned()
}

fn key_part(value: &str) -> String {
    value
        .chars()
        .map(|character| match character {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' => character,
            _ => '_',
        })
        .collect()
}

fn unix_time_ms() -> Result<u64, AppError> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| AppError::Internal(format!("system time before unix epoch: {error}")))?;

    Ok(duration.as_millis() as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_limiter_only_applies_to_mutations() {
        assert!(!is_write_method(&Method::GET));
        assert!(!is_write_method(&Method::HEAD));
        assert!(is_write_method(&Method::POST));
        assert!(is_write_method(&Method::PATCH));
        assert!(is_write_method(&Method::DELETE));
    }

    #[test]
    fn key_parts_are_redis_namespace_safe() {
        assert_eq!(key_part("/posts/:id/comments"), "_posts__id_comments");
        assert_eq!(key_part("2001:db8::1"), "2001_db8__1");
    }

    #[test]
    fn client_ip_prefers_edge_forwarded_headers() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-forwarded-for",
            axum::http::HeaderValue::from_static("203.0.113.10, 10.0.0.1"),
        );
        headers.insert(
            "x-real-ip",
            axum::http::HeaderValue::from_static("198.51.100.3"),
        );
        headers.insert(
            "fly-client-ip",
            axum::http::HeaderValue::from_static("192.0.2.7"),
        );

        assert_eq!(client_ip(&headers), "192.0.2.7");

        headers.remove("fly-client-ip");
        assert_eq!(client_ip(&headers), "198.51.100.3");

        headers.remove("x-real-ip");
        assert_eq!(client_ip(&headers), "203.0.113.10");
    }
}
