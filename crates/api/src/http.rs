use axum::routing::{get, post};
use axum::Router;
use tower_http::trace::TraceLayer;

use crate::auth::{forgot_password, login, logout, refresh, reset_password, signup, verify_email};
use crate::comments::{create_comment, delete_comment, get_post_comments};
use crate::health::healthz;
use crate::media::{complete_upload, create_upload};
use crate::posts::{create_post, delete_post, get_post, get_user_posts};
use crate::profile::{create_avatar_upload, get_user_profile, update_me};
use crate::social::{toggle_comment_like, toggle_post_like, toggle_post_save};
use crate::state::AppState;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/auth/signup", post(signup))
        .route("/auth/login", post(login))
        .route("/auth/refresh", post(refresh))
        .route("/auth/logout", post(logout))
        .route("/auth/verify-email", post(verify_email))
        .route("/auth/forgot-password", post(forgot_password))
        .route("/auth/reset-password", post(reset_password))
        .route("/media/uploads", post(create_upload))
        .route("/media/uploads/:id/complete", post(complete_upload))
        .route("/posts", post(create_post))
        .route("/posts/:id", get(get_post).delete(delete_post))
        .route("/posts/:id/like", post(toggle_post_like))
        .route("/posts/:id/save", post(toggle_post_save))
        .route(
            "/posts/:id/comments",
            get(get_post_comments).post(create_comment),
        )
        .route("/comments/:id", axum::routing::delete(delete_comment))
        .route("/comments/:id/like", post(toggle_comment_like))
        .route("/users/:handle", get(get_user_profile))
        .route("/users/:handle/posts", get(get_user_posts))
        .route("/me", axum::routing::patch(update_me))
        .route("/me/avatar", post(create_avatar_upload))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
