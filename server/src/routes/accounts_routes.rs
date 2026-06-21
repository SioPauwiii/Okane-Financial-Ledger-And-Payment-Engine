use axum::routing::{get, post};
use axum::Router;
use crate::{handlers::accounts_handlers, state::AppState};

pub fn accounts_routes() -> Router<AppState> {
	Router::new()
		.route("/my-account", get(accounts_handlers::my_account))
		.route("/deposit", post(accounts_handlers::start_deposit))
		.route("/withdraw", post(accounts_handlers::withdraw))
		.route("/transfer", post(accounts_handlers::transfer))
		.route("/webhook/paymongo", post(accounts_handlers::paymongo_webhook))
}
