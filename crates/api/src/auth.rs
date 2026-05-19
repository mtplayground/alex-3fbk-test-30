use axum::extract::State;
use axum::http::header::{COOKIE, SET_COOKIE};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::Json;
use serde::{Deserialize, Serialize};
use sqlx::error::DatabaseError;
use zeroclaw_core::auth::{self as core_auth, SignedToken};
use zeroclaw_core::models::{CreateRefreshToken, CreateUser, User, UserId};
use zeroclaw_core::repositories::{refresh_tokens, users};

use crate::error::AppError;
use crate::state::AppState;

const REFRESH_COOKIE_NAME: &str = "zc_refresh";
const REFRESH_COOKIE_MAX_AGE_SECONDS: i64 = 60 * 60 * 24 * 30;
const ACCESS_TOKEN_EXPIRES_IN_SECONDS: u64 = 60 * 15;

#[derive(Debug, Deserialize)]
pub struct SignupRequest {
    email: String,
    handle: String,
    password: String,
    display_name: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    access_token: String,
    token_type: &'static str,
    expires_in: u64,
    user: UserResponse,
}

#[derive(Debug, Serialize)]
pub struct AccessTokenResponse {
    access_token: String,
    token_type: &'static str,
    expires_in: u64,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    id: String,
    email: String,
    handle: String,
    display_name: String,
    bio: String,
    link: Option<String>,
    avatar_key: Option<String>,
    is_private: bool,
}

impl UserResponse {
    fn from_user(user: &User) -> Self {
        Self {
            id: user.id().to_string(),
            email: user.email().to_owned(),
            handle: user.handle().to_owned(),
            display_name: user.display_name().to_owned(),
            bio: user.bio().to_owned(),
            link: user.link().map(str::to_owned),
            avatar_key: user.avatar_key().map(str::to_owned),
            is_private: user.is_private(),
        }
    }
}

pub async fn signup(
    State(state): State<AppState>,
    Json(payload): Json<SignupRequest>,
) -> Result<(StatusCode, HeaderMap, Json<AuthResponse>), AppError> {
    validate_signup(&payload)?;

    let password_hash = core_auth::hash_password(&payload.password)?;
    let input = CreateUser::new(
        payload.email,
        payload.handle,
        password_hash,
        payload.display_name.trim().to_owned(),
    );

    let user = match users::create(state.db_pool(), &input).await {
        Ok(user) => user,
        Err(error) if is_unique_violation(&error) => {
            return Err(AppError::Conflict("email or handle already exists"));
        }
        Err(error) => return Err(AppError::Database(error)),
    };

    let tokens = issue_initial_tokens(&state, user.id()).await?;
    let headers = refresh_cookie_headers(tokens.refresh_token.token())?;
    let response = AuthResponse {
        access_token: tokens.access_token.token().to_owned(),
        token_type: "Bearer",
        expires_in: ACCESS_TOKEN_EXPIRES_IN_SECONDS,
        user: UserResponse::from_user(&user),
    };

    Ok((StatusCode::CREATED, headers, Json(response)))
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<(StatusCode, HeaderMap, Json<AuthResponse>), AppError> {
    validate_login(&payload)?;

    let Some(user) = users::find_by_email(state.db_pool(), &payload.email).await? else {
        return Err(AppError::Unauthorized);
    };

    if !core_auth::verify_password(&payload.password, user.password_hash())? {
        return Err(AppError::Unauthorized);
    }

    let tokens = issue_initial_tokens(&state, user.id()).await?;
    let headers = refresh_cookie_headers(tokens.refresh_token.token())?;
    let response = AuthResponse {
        access_token: tokens.access_token.token().to_owned(),
        token_type: "Bearer",
        expires_in: ACCESS_TOKEN_EXPIRES_IN_SECONDS,
        user: UserResponse::from_user(&user),
    };

    Ok((StatusCode::OK, headers, Json(response)))
}

pub async fn refresh(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<(StatusCode, HeaderMap, Json<AccessTokenResponse>), AppError> {
    let token = refresh_cookie_value(&headers).ok_or(AppError::Unauthorized)?;
    let claims =
        core_auth::verify_refresh_token(state.jwt(), &token).map_err(|_| AppError::Unauthorized)?;
    let user_id = claims.user_id().map_err(|_| AppError::Unauthorized)?;
    let token_jti = claims.jti().ok_or(AppError::Unauthorized)?;

    let Some(current_refresh) =
        refresh_tokens::find_active_by_jti(state.db_pool(), token_jti).await?
    else {
        return Err(AppError::Unauthorized);
    };

    if current_refresh.user_id() != user_id {
        return Err(AppError::Unauthorized);
    }

    let access_token = core_auth::sign_access_token(state.jwt(), user_id)?;
    let new_refresh_token = core_auth::sign_refresh_token(state.jwt(), user_id)?;
    let new_refresh_input =
        refresh_token_input(user_id, &new_refresh_token)?.rotated_from(current_refresh.id());
    refresh_tokens::rotate(state.db_pool(), current_refresh.id(), &new_refresh_input).await?;

    let headers = refresh_cookie_headers(new_refresh_token.token())?;
    let response = AccessTokenResponse {
        access_token: access_token.token().to_owned(),
        token_type: "Bearer",
        expires_in: ACCESS_TOKEN_EXPIRES_IN_SECONDS,
    };

    Ok((StatusCode::OK, headers, Json(response)))
}

pub async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<(StatusCode, HeaderMap), AppError> {
    if let Some(token) = refresh_cookie_value(&headers) {
        if let Ok(claims) = core_auth::verify_refresh_token(state.jwt(), &token) {
            if let Some(token_jti) = claims.jti() {
                if let Some(refresh_token) =
                    refresh_tokens::find_by_jti(state.db_pool(), token_jti).await?
                {
                    refresh_tokens::revoke(state.db_pool(), refresh_token.id()).await?;
                }
            }
        }
    }

    Ok((StatusCode::NO_CONTENT, clear_refresh_cookie_headers()?))
}

struct IssuedTokens {
    access_token: SignedToken,
    refresh_token: SignedToken,
}

async fn issue_initial_tokens(state: &AppState, user_id: UserId) -> Result<IssuedTokens, AppError> {
    let access_token = core_auth::sign_access_token(state.jwt(), user_id)?;
    let refresh_token = core_auth::sign_refresh_token(state.jwt(), user_id)?;
    let refresh_input = refresh_token_input(user_id, &refresh_token)?;

    refresh_tokens::create(state.db_pool(), &refresh_input).await?;

    Ok(IssuedTokens {
        access_token,
        refresh_token,
    })
}

fn refresh_token_input(
    user_id: UserId,
    signed_token: &SignedToken,
) -> Result<CreateRefreshToken, AppError> {
    let token_jti = signed_token
        .claims()
        .jti()
        .ok_or_else(|| AppError::Internal("refresh token missing jti".to_owned()))?;
    let expires_at = signed_token.claims().expires_at()?;

    Ok(CreateRefreshToken::new(user_id, token_jti, expires_at))
}

fn validate_signup(payload: &SignupRequest) -> Result<(), AppError> {
    require_non_empty("email", &payload.email)?;
    require_non_empty("handle", &payload.handle)?;
    require_non_empty("display_name", &payload.display_name)?;
    validate_password(&payload.password)
}

fn validate_login(payload: &LoginRequest) -> Result<(), AppError> {
    require_non_empty("email", &payload.email)?;
    require_non_empty("password", &payload.password)
}

fn require_non_empty(field: &'static str, value: &str) -> Result<(), AppError> {
    if value.trim().is_empty() {
        return Err(AppError::BadRequest(field));
    }

    Ok(())
}

fn validate_password(password: &str) -> Result<(), AppError> {
    if password.len() < 8 {
        return Err(AppError::BadRequest(
            "password must be at least 8 characters",
        ));
    }

    Ok(())
}

fn refresh_cookie_value(headers: &HeaderMap) -> Option<String> {
    let cookie_header = headers.get(COOKIE)?.to_str().ok()?;

    cookie_header.split(';').find_map(|cookie| {
        let (name, value) = cookie.trim().split_once('=')?;
        (name == REFRESH_COOKIE_NAME).then(|| value.to_owned())
    })
}

fn refresh_cookie_headers(token: &str) -> Result<HeaderMap, AppError> {
    let cookie = format!(
        "{REFRESH_COOKIE_NAME}={token}; Path=/auth; HttpOnly; SameSite=Lax; Max-Age={REFRESH_COOKIE_MAX_AGE_SECONDS}"
    );
    set_cookie_header(cookie)
}

fn clear_refresh_cookie_headers() -> Result<HeaderMap, AppError> {
    set_cookie_header(format!(
        "{REFRESH_COOKIE_NAME}=; Path=/auth; HttpOnly; SameSite=Lax; Max-Age=0; Expires=Thu, 01 Jan 1970 00:00:00 GMT"
    ))
}

fn set_cookie_header(cookie: String) -> Result<HeaderMap, AppError> {
    let mut headers = HeaderMap::new();
    let value =
        HeaderValue::from_str(&cookie).map_err(|error| AppError::Internal(error.to_string()))?;
    headers.insert(SET_COOKIE, value);
    Ok(headers)
}

fn is_unique_violation(error: &sqlx::Error) -> bool {
    error
        .as_database_error()
        .and_then(DatabaseError::code)
        .is_some_and(|code| code == "23505")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refresh_cookie_parser_finds_named_cookie() {
        let mut headers = HeaderMap::new();
        headers.insert(
            COOKIE,
            HeaderValue::from_static("theme=dark; zc_refresh=token-value; other=true"),
        );

        assert_eq!(
            refresh_cookie_value(&headers),
            Some("token-value".to_owned())
        );
    }

    #[test]
    fn refresh_cookie_parser_ignores_missing_cookie() {
        let mut headers = HeaderMap::new();
        headers.insert(COOKIE, HeaderValue::from_static("theme=dark"));

        assert_eq!(refresh_cookie_value(&headers), None);
    }
}
