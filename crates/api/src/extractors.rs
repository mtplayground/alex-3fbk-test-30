use axum::async_trait;
use axum::extract::FromRequestParts;
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;
use zeroclaw_core::auth as core_auth;
use zeroclaw_core::models::{User, UserId};
use zeroclaw_core::repositories::users;

use crate::error::AppError;
use crate::state::AppState;

#[derive(Debug, Clone)]
pub struct AuthUser {
    user: User,
}

impl AuthUser {
    pub fn new(user: User) -> Self {
        Self { user }
    }

    pub const fn id(&self) -> UserId {
        self.user.id()
    }

    pub const fn user(&self) -> &User {
        &self.user
    }

    pub fn into_user(self) -> User {
        self.user
    }
}

#[async_trait]
impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token = bearer_token(parts).ok_or(AppError::Unauthorized)?;
        load_auth_user(state, token).await
    }
}

#[derive(Debug, Clone)]
pub struct OptionalAuthUser(pub Option<AuthUser>);

impl OptionalAuthUser {
    pub fn as_ref(&self) -> Option<&AuthUser> {
        self.0.as_ref()
    }

    pub fn into_inner(self) -> Option<AuthUser> {
        self.0
    }
}

#[async_trait]
impl FromRequestParts<AppState> for OptionalAuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let Some(token) = bearer_token(parts) else {
            return Ok(Self(None));
        };

        load_auth_user(state, token).await.map(Some).map(Self)
    }
}

async fn load_auth_user(state: &AppState, token: &str) -> Result<AuthUser, AppError> {
    let claims =
        core_auth::verify_access_token(state.jwt(), token).map_err(|_| AppError::Unauthorized)?;
    let user_id = claims.user_id().map_err(|_| AppError::Unauthorized)?;
    let Some(user) = users::find_by_id(state.db_pool(), user_id).await? else {
        return Err(AppError::Unauthorized);
    };

    Ok(AuthUser::new(user))
}

fn bearer_token(parts: &Parts) -> Option<&str> {
    let header = parts.headers.get(AUTHORIZATION)?.to_str().ok()?;
    parse_bearer_token(header)
}

fn parse_bearer_token(header: &str) -> Option<&str> {
    let (scheme, token) = header.trim().split_once(' ')?;
    if !scheme.eq_ignore_ascii_case("Bearer") || token.trim().is_empty() {
        return None;
    }

    Some(token.trim())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bearer_parser_accepts_case_insensitive_scheme() {
        assert_eq!(parse_bearer_token("bearer token"), Some("token"));
        assert_eq!(parse_bearer_token("Bearer token"), Some("token"));
    }

    #[test]
    fn bearer_parser_rejects_missing_or_wrong_scheme() {
        assert_eq!(parse_bearer_token("token"), None);
        assert_eq!(parse_bearer_token("Basic token"), None);
        assert_eq!(parse_bearer_token("Bearer   "), None);
    }
}
