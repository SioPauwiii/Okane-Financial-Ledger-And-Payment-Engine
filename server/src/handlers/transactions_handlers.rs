use axum::{extract::State, http::StatusCode, response::{IntoResponse, Response}, Json};
use axum_extra::extract::cookie::CookieJar;
use crate::{
    errors::AppError,
    state::AppState,
    services::cookie_services::AUTH_COOKIE_NAME,
    services::transactions_services,
    responses::transaction_responses::MyTransactionsResponse,
};

pub async fn my_transactions(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<Response, AppError> {
    let token = jar
        .get(AUTH_COOKIE_NAME)
        .map(|c| c.value().to_string())
        .ok_or_else(|| AppError::Unauthorized("Missing auth token".to_string()))?;

    let response = transactions_services::my_transactions(&state, &token).await?;

    Ok((StatusCode::OK, Json(response)).into_response())
}
