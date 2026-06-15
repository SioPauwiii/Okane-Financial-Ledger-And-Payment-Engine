use axum::{routing::get, Router};
use crate::{handlers::*, state::AppState};
use crate::routes::auth_routes::auth_routes;
use crate::routes::users_routes::users_routes;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::trace::TraceLayer;

pub fn build_server(state: AppState) -> Router {
    let frontend_origin = std::env::var("FRONTEND_ORIGIN").unwrap_or_else(|_| "http://localhost:8081".into());
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::exact(frontend_origin.parse().expect("FRONTEND_ORIGIN must be a valid origin")))
        .allow_credentials(true)
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::DELETE,
            axum::http::Method::PATCH,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers([
            axum::http::header::AUTHORIZATION,
            axum::http::header::ACCEPT,
            axum::http::header::CONTENT_TYPE,
        ]);

    Router::new()
        .route("/health", get(|| async { "OK" }))
        .nest("/api/auth", auth_routes())
        .nest("/api/user", users_routes())
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

