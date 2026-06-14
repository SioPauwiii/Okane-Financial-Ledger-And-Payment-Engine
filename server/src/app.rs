use axum::{routing::{get, post, put, patch, delete}, Router};
use crate::{handlers::*, state::AppState};
use crate::routes::auth_routes::auth_routes;
use tower_http::trace::TraceLayer;

pub fn build_server(state: AppState) -> Router {
    Router::new()
        .route("/health", get(|| async { "OK" }))
        .nest("/api/auth", auth_routes())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

