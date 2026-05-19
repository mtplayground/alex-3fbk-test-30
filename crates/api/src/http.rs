use axum::routing::get;
use axum::Router;
use tower_http::trace::TraceLayer;

use crate::health::healthz;
use crate::state::AppState;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
