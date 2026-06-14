use axum::{extract::State, http::StatusCode, Json};
use crate::{
    errors::AppError,
    state::AppState, 
    services::auth_services, 
    requests::auth_requests::{AuthResponse, LoginRequest, RegisterRequest},
};

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<AuthResponse>), AppError> {
    let response = auth_services::register(&state, payload).await?;
    Ok((
        StatusCode::CREATED,
        Json(AuthResponse {
            message: "User registered successfully".to_string(),
            user: response.user,
            access_token: response.access_token,
        }),
    ))
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<(StatusCode, Json<AuthResponse>), AppError> {
    let response = auth_services::login(&state, payload).await?;
    Ok((
        StatusCode::OK,
        Json(AuthResponse {
            message: "User logged in successfully".to_string(),
            user: response.user,
            access_token: response.access_token,
        }),
    ))
}

// pub async fn logout(
//     State(state): State<AppState>,
//     Json(payload): Json<auth_requests::AuthResponse>,
// ) -> Result<(StatusCode, Json<auth_requests::LogoutResponse>), AppError> {
//     auth_services::logout(&state, payload).await?;
//     Ok((
//         StatusCode::OK,
//         Json(LogoutResponse {
//             message: "User logged out successfully".to_string(),
//         }),
//     ))
// }

// pub async fn me(
//     State(state): State<AppState>,
//     Json(payload): Json<auth_requests::AuthResponse>,
// ) -> Result<(StatusCode, Json<auth_requests::MeResponse>), AppError> {
//     let response = auth_services::me(&state, payload).await?;
//     Ok((
//         StatusCode::OK,
//         Json(MeResponse {
//             user: response.user,
//         }),
//     ))
// }