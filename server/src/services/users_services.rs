use crate::{
    errors::AppError,
    state::AppState,
    responses::user_responses::MeResponse,
    models::users_models::User,
};
use crate::responses::jwt_responses::JwtClaims;
use jsonwebtoken::{decode, DecodingKey, Validation};

fn jwt_secret() -> String {
    std::env::var("JWT_SECRET").unwrap_or_else(|_| "secret".into())
}

pub async fn me(state: &AppState, token: &str) -> Result<MeResponse, AppError> {
    let token_data = decode::<JwtClaims>(
        token,
        &DecodingKey::from_secret(jwt_secret().as_ref()),
        &Validation::default(),
    )
    .map_err(|_| AppError::Unauthorized("Invalid or expired session".to_string()))?;

    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE id = $1"
    )
    .bind(token_data.claims.sub)
    .fetch_one(&state.db)
    .await?;

    Ok(MeResponse { user })
}