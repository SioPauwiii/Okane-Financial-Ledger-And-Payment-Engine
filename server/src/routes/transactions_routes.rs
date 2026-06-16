use axum::routing::{get, post};
use axum::Router;
use crate::{handlers::transactions_handlers, state::AppState};

pub fn transactions_routes() -> Router<AppState> {
	Router::new()
		.route("/my-transactions", get(transactions_handlers::my_transactions))
}