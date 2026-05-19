use axum::extract::{Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use zeroclaw_core::repositories::search as search_repo;

use crate::error::AppError;
use crate::posts::PostResponse;
use crate::state::AppState;

const DEFAULT_SEARCH_LIMIT: i64 = 10;
const MAX_SEARCH_LIMIT: i64 = 25;
const MAX_SEARCH_QUERY_CHARS: usize = 100;

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    q: String,
    limit: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct SearchResponse {
    users: Vec<SearchUserResponse>,
    hashtags: Vec<SearchHashtagResponse>,
    posts: Vec<PostResponse>,
}

#[derive(Debug, Serialize)]
pub struct SearchUserResponse {
    id: String,
    handle: String,
    display_name: String,
    avatar_key: Option<String>,
    is_private: bool,
}

#[derive(Debug, Serialize)]
pub struct SearchHashtagResponse {
    name: String,
    post_count: i64,
}

pub async fn search(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<SearchResponse>, AppError> {
    let q = normalize_query(query.q)?;
    let limit = query
        .limit
        .unwrap_or(DEFAULT_SEARCH_LIMIT)
        .clamp(1, MAX_SEARCH_LIMIT);
    let users = search_repo::users(state.db_pool(), &q, limit).await?;
    let hashtags = search_repo::hashtags(state.db_pool(), &q, limit).await?;
    let posts = search_repo::posts_matching(state.db_pool(), &q, limit).await?;

    Ok(Json(SearchResponse {
        users: users.into_iter().map(SearchUserResponse::from).collect(),
        hashtags: hashtags
            .into_iter()
            .map(SearchHashtagResponse::from)
            .collect(),
        posts: posts.into_iter().map(PostResponse::from).collect(),
    }))
}

impl From<search_repo::UserSearchResult> for SearchUserResponse {
    fn from(user: search_repo::UserSearchResult) -> Self {
        Self {
            id: user.id.to_string(),
            handle: user.handle,
            display_name: user.display_name,
            avatar_key: user.avatar_key,
            is_private: user.is_private,
        }
    }
}

impl From<search_repo::HashtagSearchResult> for SearchHashtagResponse {
    fn from(hashtag: search_repo::HashtagSearchResult) -> Self {
        Self {
            name: hashtag.name,
            post_count: hashtag.post_count,
        }
    }
}

fn normalize_query(query: String) -> Result<String, AppError> {
    let query = query.trim().trim_start_matches('#').trim_start_matches('@');
    if query.is_empty() {
        return Err(AppError::BadRequest("q"));
    }

    if query.chars().count() > MAX_SEARCH_QUERY_CHARS {
        return Err(AppError::BadRequest("q"));
    }

    Ok(query.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_query_accepts_hashtag_and_mention_prefixes() {
        assert_eq!(normalize_query("  #rust ".to_owned()).expect("valid"), "rust");
        assert_eq!(normalize_query("@mira".to_owned()).expect("valid"), "mira");
    }

    #[test]
    fn normalize_query_rejects_empty_values() {
        assert!(matches!(
            normalize_query("   ".to_owned()),
            Err(AppError::BadRequest("q"))
        ));
    }
}
