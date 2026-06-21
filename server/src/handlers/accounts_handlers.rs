use axum::{extract::State, http::StatusCode, response::{IntoResponse, Response}, Json};
use axum_extra::extract::cookie::CookieJar;
use axum::body::Bytes;
use axum::http::HeaderMap;
use rust_decimal::Decimal;
use crate::{
    errors::AppError,
    state::AppState,
    services::accounts_services,
    services::paymongo_services,
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

pub async fn paymongo_webhook(
    State(state): State<AppState>,
    header: HeaderMap,
    body: Bytes,
) -> Result<Response, AppError> {
    let sig_header = header.get("paymongo-signature")
    .and_then(|v| v.to_str().ok())
    .ok_or_else(|| AppError::Unauthorized("Missing paymongo-signature header".to_string()))?;

    let is_valid = paymongo_services::verify_webhook_signature(&body, sig_header, &state.paymongo_webhook_secret);

    if !is_valid {
        return Err(AppError::Unauthorized("Invalid webhook signature".to_string()));    
    }

    let payload : serde_json::Value = serde_json::from_slice(&body)
    .map_err(|_| AppError::BadRequest("Invalid JSON payload".to_string()))?;

    let event_type = payload["data"]["attributes"]["type"]
    .as_str()
    .unwrap_or("");

    if event_type != "checkout_session.payment.paid" {
        return Ok((StatusCode::OK,Json(serde_json::json!({
            "received": true,
            "event_type": event_type
        }))).into_response());
    }

    let checkout_data = &payload["data"]["attributes"]["data"]["attributes"];

    let account_number = checkout_data["metadata"]["account_number"]
        .as_str()
        .ok_or_else(|| AppError::InternalServerError("Missing account_number in checkout metadata".to_string()))?;

    let transaction_uuid_str = checkout_data["metadata"]["transaction_uuid"]
        .as_str()
        .ok_or_else(|| AppError::InternalServerError("Missing transaction_uuid in checkout metadata".to_string()))?;

    let transaction_uuid = uuid::Uuid::parse_str(transaction_uuid_str)
        .map_err(|_| AppError::InternalServerError("Invalid transaction_uuid in checkout metadata".to_string()))?;

    let amount_in_centavos = checkout_data["line_items"][0]["amount"]
        .as_u64()
        .ok_or_else(|| AppError::InternalServerError("Missing amount in checkout line_items".to_string()))?;

    let amount = Decimal::new(amount_in_centavos as i64, 2);
    accounts_services::complete_deposit(&state, account_number, amount, transaction_uuid).await?;

    Ok((StatusCode::OK, Json(serde_json::json!({
        "received": true
    }))).into_response())
}