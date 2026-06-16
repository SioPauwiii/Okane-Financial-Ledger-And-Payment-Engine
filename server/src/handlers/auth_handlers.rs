use axum::{extract::State, http::{header::SET_COOKIE, HeaderValue, StatusCode}, response::{IntoResponse, Response}, Json};
use crate::{
    errors::AppError,
    state::AppState, 
    services::auth_services::{self}, 
    requests::auth_requests::{LoginRequest, RegisterRequest},
    services::cookie_services,
};

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Response, AppError> {
    let response = auth_services::register(&state, payload).await?;
    let cookie = response
        .access_token
        .as_deref()
        .map(cookie_services::build_auth_cookie)
        .ok_or_else(|| AppError::InternalServerError("Missing access token".to_string()))?;
    let cookie_value = HeaderValue::from_str(&cookie.to_string())
        .map_err(|_| AppError::InternalServerError("Failed to set auth cookie".to_string()))?;

    Ok((
        StatusCode::CREATED,
        [(SET_COOKIE, cookie_value)],
        Json(response),
    )
    .into_response())
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Response, AppError> {
    let response = auth_services::login(&state, payload).await?;
    let cookie = cookie_services::build_auth_cookie(&response.access_token);
    let cookie_value = HeaderValue::from_str(&cookie.to_string())
        .map_err(|_| AppError::InternalServerError("Failed to set auth cookie".to_string()))?;

    Ok((
        StatusCode::OK,
        [(SET_COOKIE, cookie_value)],
        Json(response),
    )
    .into_response())
}

// pub async fn logout(
//     State(state): State<AppState>,
//     Json(payload): Json<auth_requests::RegisterResponse>,
// ) -> Result<(StatusCode, Json<auth_requests::LogoutResponse>), AppError> {
//     auth_services::logout(&state, payload).await?;
//     Ok((
//         StatusCode::OK,
//         Json(LogoutResponse {
//             message: "User logged out successfully".to_string(),
//         }),
//     ))
// }