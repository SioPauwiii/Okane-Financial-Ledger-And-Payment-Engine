use axum::{extract::State, http::StatusCode, response::{IntoResponse, Response}, Json};
use axum_extra::extract::cookie::CookieJar;
use crate::{
    errors::AppError,
    state::AppState,
    services::accounts_services,
    services::cookie_services::AUTH_COOKIE_NAME,
    requests::account_requests::{DepositRequest, WithdrawalRequest, TransferRequest},
};

pub async fn my_account(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<Response, AppError> {
    let token = jar
        .get(AUTH_COOKIE_NAME)
        .map(|c| c.value().to_string())
        .ok_or_else(|| AppError::Unauthorized("Missing auth token".to_string()))?;

    let response = accounts_services::my_account(&state, &token).await?;

    Ok((StatusCode::OK, Json(response)).into_response())
}

pub async fn start_deposit(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(payload): Json<DepositRequest>,
) -> Result<Response, AppError> {
    let token = jar
        .get(AUTH_COOKIE_NAME)
        .map(|c| c.value().to_string())
        .ok_or_else(|| AppError::Unauthorized("Missing auth token".to_string()))?;

    let response = accounts_services::start_deposit(&state, &token, payload.amount, &payload.account_number).await?;

    Ok((StatusCode::OK, Json(response)).into_response())
}

pub async fn withdraw(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(payload): Json<WithdrawalRequest>,
) -> Result<Response, AppError> {
    let token = jar
        .get(AUTH_COOKIE_NAME)
        .map(|c| c.value().to_string())
        .ok_or_else(|| AppError::Unauthorized("Missing auth token".to_string()))?;

    let response = accounts_services::withdraw(&state, &token, payload.amount, &payload.account_number).await?;

    Ok((StatusCode::OK, Json(response)).into_response())
}

pub async fn transfer(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(payload): Json<TransferRequest>,
) -> Result<Response, AppError> {
    let token = jar
        .get(AUTH_COOKIE_NAME)
        .map(|c| c.value().to_string())
        .ok_or_else(|| AppError::Unauthorized("Missing auth token".to_string()))?;

    let response = accounts_services::transfer(&state, &token, payload.amount, &payload.target_account_number).await?;

    Ok((StatusCode::OK, Json(response)).into_response())
}

