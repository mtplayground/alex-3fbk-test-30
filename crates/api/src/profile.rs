use std::time::Duration;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use zeroclaw_core::models::{UpdateUserProfile, User};
use zeroclaw_core::repositories::{moderation, users};

use crate::error::AppError;
use crate::extractors::{AuthUser, OptionalAuthUser};
use crate::state::AppState;

const AVATAR_UPLOAD_EXPIRES_SECONDS: u64 = 15 * 60;

#[derive(Debug, Serialize)]
pub struct ProfileResponse {
    id: String,
    email: Option<String>,
    handle: String,
    display_name: String,
    bio: String,
    link: Option<String>,
    avatar_key: Option<String>,
    is_private: bool,
    email_verified: bool,
}

impl ProfileResponse {
    fn public(user: &User) -> Self {
        Self::from_user(user, false)
    }

    fn private(user: &User) -> Self {
        Self::from_user(user, true)
    }

    fn from_user(user: &User, include_email: bool) -> Self {
        Self {
            id: user.id().to_string(),
            email: include_email.then(|| user.email().to_owned()),
            handle: user.handle().to_owned(),
            display_name: user.display_name().to_owned(),
            bio: user.bio().to_owned(),
            link: user.link().map(str::to_owned),
            avatar_key: user.avatar_key().map(str::to_owned),
            is_private: user.is_private(),
            email_verified: user.email_verified_at().is_some(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    display_name: Option<String>,
    bio: Option<String>,
    link: Option<Option<String>>,
    is_private: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct AvatarUploadRequest {
    content_type: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AvatarUploadResponse {
    key: String,
    upload_url: String,
    method: &'static str,
    expires_in: u64,
    user: ProfileResponse,
}

pub async fn get_user_profile(
    State(state): State<AppState>,
    OptionalAuthUser(auth_user): OptionalAuthUser,
    Path(handle): Path<String>,
) -> Result<Json<ProfileResponse>, AppError> {
    require_non_empty("handle", &handle)?;

    let Some(user) = users::find_by_handle(state.db_pool(), &handle).await? else {
        return Err(AppError::NotFound);
    };
    if let Some(auth_user) = auth_user {
        if auth_user.id() != user.id()
            && moderation::is_blocked_between(state.db_pool(), auth_user.id(), user.id()).await?
        {
            return Err(AppError::NotFound);
        }
    }

    Ok(Json(ProfileResponse::public(&user)))
}

pub async fn update_me(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<UpdateProfileRequest>,
) -> Result<Json<ProfileResponse>, AppError> {
    let input = update_profile_input(payload)?;
    let user = users::update_profile(state.db_pool(), auth_user.id(), &input).await?;

    Ok(Json(ProfileResponse::private(&user)))
}

pub async fn create_avatar_upload(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<AvatarUploadRequest>,
) -> Result<(StatusCode, Json<AvatarUploadResponse>), AppError> {
    let content_type = payload
        .content_type
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let extension = avatar_extension(content_type);
    let key = format!(
        "avatars/{}/{}.{}",
        auth_user.id(),
        Uuid::new_v4(),
        extension
    );
    let expires_in = Duration::from_secs(AVATAR_UPLOAD_EXPIRES_SECONDS);
    let presigned = state
        .storage()
        .presigned_put(&key, expires_in, content_type)
        .await?;
    let user = users::update_avatar_key(state.db_pool(), auth_user.id(), &key).await?;
    let response = AvatarUploadResponse {
        key,
        upload_url: presigned.url().to_owned(),
        method: presigned.method(),
        expires_in: presigned.expires_in().as_secs(),
        user: ProfileResponse::private(&user),
    };

    Ok((StatusCode::CREATED, Json(response)))
}

fn update_profile_input(payload: UpdateProfileRequest) -> Result<UpdateUserProfile, AppError> {
    let mut input = UpdateUserProfile::new();

    if let Some(display_name) = payload.display_name {
        let display_name = require_owned_non_empty("display_name", display_name)?;
        input = input.with_display_name(display_name);
    }

    if let Some(bio) = payload.bio {
        input = input.with_bio(bio.trim().to_owned());
    }

    if let Some(link) = payload.link {
        let normalized = match link {
            Some(value) => {
                let value = value.trim().to_owned();
                if value.is_empty() {
                    None
                } else {
                    Some(value)
                }
            }
            None => None,
        };
        input = input.with_link(normalized);
    }

    if let Some(is_private) = payload.is_private {
        input = input.with_is_private(is_private);
    }

    Ok(input)
}

fn require_non_empty(field: &'static str, value: &str) -> Result<(), AppError> {
    if value.trim().is_empty() {
        return Err(AppError::BadRequest(field));
    }

    Ok(())
}

fn require_owned_non_empty(field: &'static str, value: String) -> Result<String, AppError> {
    let value = value.trim().to_owned();
    if value.is_empty() {
        return Err(AppError::BadRequest(field));
    }

    Ok(value)
}

fn avatar_extension(content_type: Option<&str>) -> &'static str {
    match content_type {
        Some("image/png") => "png",
        Some("image/webp") => "webp",
        Some("image/gif") => "gif",
        _ => "jpg",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn avatar_extension_uses_content_type_when_known() {
        assert_eq!(avatar_extension(Some("image/png")), "png");
        assert_eq!(avatar_extension(Some("image/webp")), "webp");
        assert_eq!(avatar_extension(Some("image/jpeg")), "jpg");
        assert_eq!(avatar_extension(None), "jpg");
    }

    #[test]
    fn empty_display_name_is_rejected() {
        let result = update_profile_input(UpdateProfileRequest {
            display_name: Some("   ".to_owned()),
            bio: None,
            link: None,
            is_private: None,
        });

        assert!(matches!(result, Err(AppError::BadRequest("display_name"))));
    }
}
