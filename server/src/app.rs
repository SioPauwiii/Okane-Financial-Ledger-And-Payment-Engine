use axum::{routing::{get, post, put, patch, delete}, Router};
use crate::{handlers::*, routes::*, state::AppState};

pub fn build_server(state: AppState) -> Router {
    Router::new()
        .route("/health", get(|| async { "OK" }))
        .with_state(state)
}

