use axum::routing::{get, post};
use axum::Router;
use tower_http::trace::TraceLayer;

use crate::auth::{login, logout, refresh, signup};
use crate::health::healthz;
use crate::state::AppState;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/auth/signup", post(signup))
        .route("/auth/login", post(login))
        .route("/auth/refresh", post(refresh))
        .route("/auth/logout", post(logout))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
