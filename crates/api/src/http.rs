use axum::routing::{get, post};
use axum::Router;
use tower_http::trace::TraceLayer;

use crate::auth::{forgot_password, login, logout, refresh, reset_password, signup, verify_email};
use crate::health::healthz;
use crate::media::{complete_upload, create_upload};
use crate::profile::{create_avatar_upload, get_user_profile, update_me};
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
        .route("/users/:handle", get(get_user_profile))
        .route("/me", axum::routing::patch(update_me))
        .route("/me/avatar", post(create_avatar_upload))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
