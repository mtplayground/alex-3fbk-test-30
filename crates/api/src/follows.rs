use axum::extract::{Path, State};
use axum::Json;
use serde::Serialize;
use uuid::Uuid;
use zeroclaw_core::models::UserId;
use zeroclaw_core::repositories::{follows, users};

use crate::error::AppError;
use crate::extractors::AuthUser;
use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct FollowResponse {
    follower_id: String,
    followee_id: String,
    state: &'static str,
}

#[derive(Debug, Serialize)]
pub struct FollowUsersResponse {
    users: Vec<FollowUserResponse>,
}

#[derive(Debug, Serialize)]
pub struct FollowUserResponse {
    id: String,
    handle: String,
    display_name: String,
    avatar_key: Option<String>,
    is_private: bool,
}

pub async fn follow_user(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(handle): Path<String>,
) -> Result<Json<FollowResponse>, AppError> {
    let Some(followee) = users::find_by_handle(state.db_pool(), &handle).await? else {
        return Err(AppError::NotFound);
    };

    if followee.id() == auth_user.id() {
        return Err(AppError::BadRequest("handle"));
    }

    let state_value = if followee.is_private() {
        follows::FollowState::Pending
    } else {
        follows::FollowState::Accepted
    };
    let follow =
        follows::upsert(state.db_pool(), auth_user.id(), followee.id(), state_value).await?;

    Ok(Json(FollowResponse::from(follow)))
}

pub async fn unfollow_user(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(handle): Path<String>,
) -> Result<Json<FollowResponse>, AppError> {
    let Some(followee) = users::find_by_handle(state.db_pool(), &handle).await? else {
        return Err(AppError::NotFound);
    };

    if followee.id() == auth_user.id() {
        return Err(AppError::BadRequest("handle"));
    }

    follows::delete(state.db_pool(), auth_user.id(), followee.id()).await?;

    Ok(Json(FollowResponse {
        follower_id: auth_user.id().to_string(),
        followee_id: followee.id().to_string(),
        state: "none",
    }))
}

pub async fn accept_follow_request(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(follower_id): Path<Uuid>,
) -> Result<Json<FollowResponse>, AppError> {
    let follower_id = UserId::from(follower_id);
    let Some(follow) = follows::accept(state.db_pool(), follower_id, auth_user.id()).await? else {
        return Err(AppError::NotFound);
    };

    Ok(Json(FollowResponse::from(follow)))
}

pub async fn reject_follow_request(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(follower_id): Path<Uuid>,
) -> Result<Json<FollowResponse>, AppError> {
    let follower_id = UserId::from(follower_id);
    let rejected = follows::reject(state.db_pool(), follower_id, auth_user.id()).await?;
    if !rejected {
        return Err(AppError::NotFound);
    }

    Ok(Json(FollowResponse {
        follower_id: follower_id.to_string(),
        followee_id: auth_user.id().to_string(),
        state: "none",
    }))
}

pub async fn get_followers(
    State(state): State<AppState>,
    Path(handle): Path<String>,
) -> Result<Json<FollowUsersResponse>, AppError> {
    let Some(user) = users::find_by_handle(state.db_pool(), &handle).await? else {
        return Err(AppError::NotFound);
    };

    let users = follows::list_followers(state.db_pool(), user.id()).await?;
    Ok(Json(FollowUsersResponse {
        users: users.into_iter().map(FollowUserResponse::from).collect(),
    }))
}

pub async fn get_following(
    State(state): State<AppState>,
    Path(handle): Path<String>,
) -> Result<Json<FollowUsersResponse>, AppError> {
    let Some(user) = users::find_by_handle(state.db_pool(), &handle).await? else {
        return Err(AppError::NotFound);
    };

    let users = follows::list_following(state.db_pool(), user.id()).await?;
    Ok(Json(FollowUsersResponse {
        users: users.into_iter().map(FollowUserResponse::from).collect(),
    }))
}

impl From<follows::Follow> for FollowResponse {
    fn from(follow: follows::Follow) -> Self {
        Self {
            follower_id: follow.follower_id.to_string(),
            followee_id: follow.followee_id.to_string(),
            state: follow.state.as_str(),
        }
    }
}

impl From<follows::FollowUser> for FollowUserResponse {
    fn from(user: follows::FollowUser) -> Self {
        Self {
            id: user.id.to_string(),
            handle: user.handle,
            display_name: user.display_name,
            avatar_key: user.avatar_key,
            is_private: user.is_private,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn follow_state_exposes_storage_value() {
        assert_eq!(follows::FollowState::Accepted.as_str(), "accepted");
        assert_eq!(follows::FollowState::Pending.as_str(), "pending");
    }
}
