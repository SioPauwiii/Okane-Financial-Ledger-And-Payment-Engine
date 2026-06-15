use axum::{extract::State, http::StatusCode, Json};
use axum_extra::extract::CookieJar;
use crate::{
    errors::AppError,
    state::AppState, 
    services::users_services, 
    services::cookie_services::AUTH_COOKIE_NAME,
    responses::user_responses::MeResponse,
};

pub async fn me(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<(StatusCode, Json<MeResponse>), AppError> {
    let token = jar
        .get(AUTH_COOKIE_NAME)
        .ok_or_else(|| AppError::Unauthorized("Not authenticated".to_string()))?
        .value()
        .to_string();

    let response = users_services::me(&state, &token).await?;
    Ok((
        StatusCode::OK,
        Json(response),
    ))
}