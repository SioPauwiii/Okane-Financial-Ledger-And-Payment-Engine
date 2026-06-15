use axum::routing::get;
use axum::Router;
use crate::{handlers::users_handlers, state::AppState};

pub fn users_routes() -> Router<AppState> {
	Router::new()
		.route("/me", get(users_handlers::me))
}