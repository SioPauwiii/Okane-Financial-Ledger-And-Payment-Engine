use crate::{
    errors::AppError,
    state::AppState,
    models::accounts_models::Account,
    models::users_models::User,
    responses::account_responses::MyAccountResponse,
    responses::jwt_responses::JwtClaims,
};
use jsonwebtoken::{decode, DecodingKey, Validation};

fn jwt_secret() -> String {
    std::env::var("JWT_SECRET").unwrap_or_else(|_| "secret".into())
}

pub async fn check_if_user_exists(
    state: &AppState,
    email: &str,
) -> Result<i32, AppError> {
    let user = sqlx::query!(
        "SELECT id FROM users WHERE email = $1",
        email
    )
    .fetch_optional(&state.db)
    .await?;

    if let Some(user) = user {
        Ok(user.id as i32)
    } else {
        Err(AppError::NotFound("User not found".into()))
    }
}

pub async fn check_if_user_account_exists(
    state: &AppState,
    user_id: i32,
) -> Result<(), AppError> {
    let account = sqlx::query!(
        "SELECT id FROM accounts WHERE user_id = $1",
        user_id
    )
    .fetch_optional(&state.db)
    .await?;
    
    if account.is_some() {
        return Err(AppError::NotFound("Account already exists".into()));
    }
    
    Ok(())
}

pub async fn create_account(
    state: &AppState,
    email: &str,
    account_type: &str,
) -> Result<Account, AppError> {
    let user_id = check_if_user_exists(state, email).await?;
    check_if_user_account_exists(state, user_id).await?;
    
    // Generate a 10-digit account number
    let account_number: String = (0..10).map(|_| (rand::random::<u8>() % 10).to_string()).collect();

    let account = sqlx::query_as!(
        Account,
        r#"
        INSERT INTO accounts (user_id, account_type, currency, status, account_number)
        VALUES ($1, $2, 'PHP', 'active', $3)
        RETURNING 
            id, 
            user_id as "user_id!", 
            account_number, 
            account_type, 
            currency, 
            balance as "balance!", 
            status, 
            created_at, 
            updated_at
        "#,
        user_id,
        account_type,
        account_number
    )
    .fetch_one(&state.db)
    .await?;
    
    Ok(account)
}

pub async fn my_account(
    state: &AppState,
    token: &str
) -> Result<MyAccountResponse, AppError> {
    let token_data = decode::<JwtClaims>(
        token,
        &DecodingKey::from_secret(jwt_secret().as_ref()),
        &Validation::default(),
    )
    .map_err(|_| AppError::Unauthorized("Invalid or expired session".to_string()))?;

    let record = sqlx::query_as::<_, UserWithAccountRow>(
        r#"
        SELECT 
            u.*,
            CASE WHEN a.id IS NOT NULL THEN row_to_json(a) ELSE NULL END as account
        FROM users u
        LEFT JOIN accounts a ON u.id = a.user_id
        WHERE u.id = $1
        "#
    )
    .bind(token_data.claims.sub as i64)
    .fetch_optional(&state.db)
    .await?;

    let user_account = match record {
        Some(r) => {
            if let Some(sqlx::types::Json(account)) = r.account {
                use crate::responses::account_responses::UserAccountDetails;
                Some(UserAccountDetails {
                    email: r.user.email,
                    first_name: r.user.first_name,
                    last_name: r.user.last_name,
                    country: r.user.country,
                    province: r.user.province,
                    city: r.user.city,
                    street: r.user.street,
                    house_no: r.user.house_no,
                    zip_code: r.user.zip_code,
                    sex: r.user.sex,
                    nationality: r.user.nationality,
                    birth_date: r.user.birth_date,
                    contact_no: r.user.contact_no,
                    user_type: r.user.user_type,

                    account_number: account.account_number,
                    account_type: account.account_type,
                    currency: account.currency,
                    balance: account.balance,
                    status: account.status,
                })
            } else {
                None // User exists, but account doesn't
            }
        },
        None => return Err(AppError::NotFound("User not found".to_string()))
    };

    Ok(MyAccountResponse { user_account })
}