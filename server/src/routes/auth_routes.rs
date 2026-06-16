use axum::routing::post;
use axum::Router;
use crate::{handlers::auth_handlers, state::AppState};

pub fn auth_routes() -> Router<AppState> {
    Router::new()
        .route("/register", post(auth_handlers::register))
        .route("/login", post(auth_handlers::login))
        // .route("/logout", post(auth_handlers::logout))
        // .route("/refresh", post(auth_handlers::refresh_token))
        // .route("/me", get(auth_handlers::get_me))

}