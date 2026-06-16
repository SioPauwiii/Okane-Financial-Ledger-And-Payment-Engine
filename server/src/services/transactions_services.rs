use crate::{
    errors::AppError,
    state::AppState,
    models::transactions_models::Transaction,
    responses::jwt_responses::JwtClaims,
};
use rust_decimal::Decimal;
use chrono::NaiveDateTime;
use jsonwebtoken::{decode, DecodingKey, Validation};

fn jwt_secret() -> String {
    std::env::var("JWT_SECRET").unwrap_or_else(|_| "secret".into())
}

pub async fn record_transaction(
    state: &AppState,
    _token: &str,
    amount_transferred: Decimal,
    from_account_number: Option<String>,
    to_account_number: Option<String>,
    transaction_type: String,
    status: String,
    created_at: NaiveDateTime,
) -> Result<Transaction, AppError> {
    
    let transaction = sqlx::query_as!(
        Transaction,
        r#"
        INSERT INTO transactions (
            amount_transferred,
            from_account_number,
            to_account_number,
            transaction_type,
            status,
            created_at
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING 
            id,
            amount_transferred as "amount_transferred!",
            from_account_number,
            to_account_number,
            transaction_type,
            status,
            created_at
        "#,
        amount_transferred,
        from_account_number,
        to_account_number,
        transaction_type,
        status,
        created_at,
    )
    .fetch_one(&state.db)
    .await?;
    
    Ok(transaction)
}

pub async fn my_transactions(
    state: &AppState,
    token: &str,
) -> Result<Vec<Transaction>, AppError> {
    let token_data = decode::<JwtClaims>(
        token,
        &DecodingKey::from_secret(jwt_secret().as_ref()),
        &Validation::default(),
    )
    .map_err(|_| AppError::Unauthorized("Invalid or expired session".to_string()))?;

    let user_account_number = token_data.claims.account_number;

    let transactions = sqlx::query_as!(
        Transaction,
        "SELECT * FROM transactions WHERE from_account_number = $1 OR to_account_number = $1 ORDER BY created_at DESC",
        user_account_number,
    )
    .fetch_all(&state.db)
    .await?;
    
    Ok(transactions)
}