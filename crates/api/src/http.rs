use axum::routing::{get, post};
use axum::Router;
use tower_http::trace::TraceLayer;

use crate::auth::{forgot_password, login, logout, refresh, reset_password, signup, verify_email};
use crate::health::healthz;
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
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
