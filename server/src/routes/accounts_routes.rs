use axum::routing::get;
use axum::Router;
use crate::{handlers::accounts_handlers, state::AppState};

pub fn accounts_routes() -> Router<AppState> {
	Router::new()
		.route("/my-account", get(accounts_handlers::my_account))
}

