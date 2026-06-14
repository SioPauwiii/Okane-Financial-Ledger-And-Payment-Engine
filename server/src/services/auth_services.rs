use crate::{
    errors::AppError,
    state::AppState,
    requests::auth_requests::{LoginRequest, RegisterRequest},
    responses::auth_responses::{RegisterResponse, LoginResponse},
    responses::jwt_responses::JwtClaims,
    models::users_models::User,
};
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::NaiveDate;
use jsonwebtoken::{encode, EncodingKey, Header};

// user checking and validation
pub async fn check_email_exists(
    state: &AppState, 
    email: &str,
) -> Result<bool, AppError> {
    let existing = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM users WHERE email = $1"
    )
    .bind(email)
    .fetch_one(&state.db)
    .await?;

    Ok(existing > 0)
}

pub async fn validate_age(
    birth_date: Option<NaiveDate>,
) -> Result<(), AppError> {
    if let Some(birth_date) = birth_date {
        let today = chrono::Utc::now().date_naive();
        let age = today.signed_duration_since(birth_date).num_days() / 365;
        if age < 18 {
            return Err(AppError::BadRequest("User must be at least 18 years old".to_string()));
        }
    }
    Ok(())
}   

// password hashing and verification
pub async fn confirm_password(
    password: &str, 
    password_confirm: &str,
) -> Result<(), AppError> {
    if password != password_confirm {
        return Err(AppError::BadRequest("Passwords do not match".to_string()));
    }
    Ok(())
}

pub async fn verify_password(
    password: &str, 
    hashed_password: &str,
) -> Result<bool, AppError> {
    let parsed_hash = PasswordHash::new(hashed_password)
        .map_err(|_| AppError::BadRequest("Invalid password hash".to_string()))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

pub async fn generate_salt() -> SaltString {
    let bytes: [u8; 16] = rand::random();
    SaltString::encode_b64(&bytes).expect("failed to create salt")
}

pub async fn hash_password(
    password: &str,
) -> Result<String, AppError> {
    let salt = generate_salt().await;
    let hashed = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|_| AppError::InternalServerError("Failed to hash password".to_string()))?
        .to_string();
    Ok(hashed)
}

// JWT generation
pub async fn generate_jwt(
    user: &User,
) -> Result<String, AppError> {
    let claims = JwtClaims {
        sub: user.id,
        email: user.email.clone(),
        user_type: user.user_type.clone(),
        exp: (chrono::Utc::now().timestamp() + 604800) as usize,
    };
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "secret".into());
    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_ref()))
        .map_err(|_| AppError::InternalServerError("Failed to generate JWT".to_string()))
}

// user registration and authentication
pub async fn register(
    state: &AppState, 
    payload: RegisterRequest,
) -> Result<RegisterResponse, AppError> {
    // validate input
    if check_email_exists(state, &payload.email).await? {
        return Err(AppError::BadRequest("Email already exists".to_string()));
    }
    validate_age(payload.birth_date).await?;
    confirm_password(&payload.password, &payload.confirm_password).await?;

    // hash password
    let hashed_password = hash_password(&payload.password).await?;

    // create user in database
    let user = sqlx::query_as::<_, User>(
        "INSERT INTO users (email, last_name, first_name, 
        country, city, street, house_no, zip_code, contact_no, 
        birth_date, sex, nationality, password, user_type)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, 'customer')
        RETURNING *"
    )
    .bind(&payload.email)
    .bind(&payload.last_name)
    .bind(&payload.first_name)
    .bind(&payload.country)
    .bind(&payload.city)
    .bind(&payload.street)
    .bind(&payload.house_no)
    .bind(&payload.zip_code)
    .bind(&payload.contact_no)
    .bind(payload.birth_date)
    .bind(&payload.sex)
    .bind(&payload.nationality)
    .bind(&hashed_password)
    .fetch_one(&state.db)
    .await?;

    let access_token = generate_jwt(&user).await?;

    Ok(RegisterResponse {
        message: "User registered successfully".to_string(),
        user: Some(user),
        access_token: Some(access_token),
    })
}

pub async fn login(
    state: &AppState,
    payload: LoginRequest,
) -> Result<LoginResponse, AppError> {
    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE email = $1"
    )
    .bind(&payload.email)
    .fetch_one(&state.db)
    .await?;

    let is_valid = verify_password(&payload.password, &user.password).await?;
    if !is_valid {
        return Err(AppError::Unauthorized("Invalid email or password".to_string()));
    }

    let access_token = generate_jwt(&user).await?;
    
    Ok(LoginResponse { 
        message: "Login successful".to_string(),
        access_token 
    })
}