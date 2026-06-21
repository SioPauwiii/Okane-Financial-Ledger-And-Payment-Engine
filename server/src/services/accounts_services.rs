use crate::{
    errors::AppError,
    state::AppState,
    models::accounts_models::Account,
    responses::account_responses::{MyAccountResponse, UserWithAccountRow},
    responses::jwt_responses::JwtClaims,
    responses::account_responses::{DepositResponse, WithdrawalResponse, TransferResponse},
    services::transactions_services,
    services::paymongo_services,

};
use jsonwebtoken::{decode, DecodingKey, Validation};
use rust_decimal::Decimal;
use serde_json::{json, Value};

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

pub async fn check_if_account_active(
    state: &AppState,
    user_id: &i32,
) -> Result<(), AppError> {
    let account = sqlx::query!(
        "SELECT id FROM accounts WHERE user_id = $1 AND status = 'active'",
        user_id
    )
    .fetch_optional(&state.db)
    .await?;
    
    if account.is_none() {
        return Err(AppError::NotFound("Account not found or inactive".into()));
    }
    
    Ok(())
}

pub async fn check_amount_being_transferred(
    _state: &AppState,
    amount: &Decimal,
) -> Result<(), AppError> {
    if amount <= &Decimal::ZERO {
        return Err(AppError::BadRequest("Amount must be greater than zero".into()));
    }
    
    Ok(())
}

pub async fn compute_for_balance(
    state: &AppState,
    account_number: &String,
) -> Result<Decimal, AppError>{
    let transactions = sqlx::query!(
        "SELECT * FROM transactions WHERE (from_account_number = $1 OR to_account_number = $1) AND status = 'completed'",
        account_number,
    )
    .fetch_all(&state.db)
    .await?;
    
    let mut balance = Decimal::ZERO;
    for transaction in transactions {
        if transaction.from_account_number == Some(account_number.to_string()) {
            balance -= transaction.amount_transferred;
        } else {
            balance += transaction.amount_transferred;
        }
    }

    Ok(balance)
}

pub async fn record_balance(
    state: &AppState,
    account_number: &String,
    balance: &Decimal,
) -> Result<(), AppError> {
    let account = sqlx::query!(
        "UPDATE accounts SET balance = $1 WHERE account_number = $2",
        balance,
        account_number,
    )
    .execute(&state.db)
    .await?;
    
    if account.rows_affected() == 0 {
        return Err(AppError::NotFound("Account not found".into()));
    }
    
    Ok(())
}

// pub async fn check_balance_and_transaction_latest(
//     state: &AppState,
//     account_number: &String,
// ) -> Result<Decimal, AppError> {
//     let transaction = sqlx::query!(
//         "SELECT * FROM transactions WHERE from_account_number = $1 OR to_account_number = $1 ORDER BY created_at DESC LIMIT 1",
//         account_number,
//     )
//     .fetch_optional(&state.db)
//     .await?;
    
//     if let Some(transaction) = transaction {
//         if transaction.from_account_number == Some(account_number.to_string()) {
//             Ok(transaction.amount_transferred)
//         } else {
//             Ok(transaction.amount_transferred)
//         }
//     } else {
//         Ok(Decimal::ZERO)
//     }
// }

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

    let user_account_number = token_data.claims.account_number;

    let balance = compute_for_balance(state, &user_account_number).await?;
    record_balance(state, &user_account_number, &balance).await?;
    
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

pub async fn start_deposit(
    state: &AppState,
    token: &str,
    amount: Decimal,
    account_number: &str,
) -> Result<DepositResponse, AppError> {
    
    let token_data = decode::<JwtClaims>(
        token,
        &DecodingKey::from_secret(jwt_secret().as_ref()),
        &Validation::default(),
    )
    .map_err(|_| AppError::Unauthorized("Invalid or expired session".to_string()))?;
    let user_id = token_data.claims.sub as i32;
    
    check_if_account_active(state, &user_id).await?;
    check_amount_being_transferred(state, &amount).await?;

    // Create the pending transaction first so we have its UUID to embed in the checkout session.
    // The webhook will read that UUID back and use it when recording the completed record.
    let transaction = transactions_services::record_transaction(
        state,
        token,
        amount,
        None,
        Some(account_number.to_string()),
        "deposit".to_string(),
        "pending".to_string(),
        chrono::Utc::now().naive_utc(),
        None, // let the DB generate a fresh UUID
    )
    .await?;

    let checkout_url = paymongo_services::create_checkout_session(
        &state.http_client,
        &state.paymongo_secret_key,
        amount,
        account_number,
        &transaction.transaction_uuid.to_string(), // embed UUID so webhook can retrieve it
    )
    .await?;

    Ok(DepositResponse {
        message: "Deposit initiated. Complete payment at the provided URL.".to_string(),
        checkout_url: Some(checkout_url),
        transaction: Some(transaction),
    })
}

pub async fn complete_deposit(
    state: &AppState,
    account_number: &str,
    amount: Decimal,
    transaction_uuid: uuid::Uuid, // UUID of the original pending transaction
) -> Result<(), AppError> {
    // Called by the PayMongo webhook after payment is confirmed.
    // We reuse the same UUID as the pending row so both rows are linked.
    let _transaction = transactions_services::record_transaction(
        state,
        "", // webhook has no user JWT token, so we pass empty
        amount,
        None,
        Some(account_number.to_string()),
        "deposit".to_string(),
        "completed".to_string(),
        chrono::Utc::now().naive_utc(),
        Some(transaction_uuid), // reuse the pending transaction's UUID
    )
    .await?;

    Ok(())
}

pub async fn withdraw(
    state: &AppState,
    token: &str,
    amount: Decimal,
    account_number: &str,
) -> Result<WithdrawalResponse, AppError> {
    
    let token_data = decode::<JwtClaims>(
        token,
        &DecodingKey::from_secret(jwt_secret().as_ref()),
        &Validation::default(),
    )
    .map_err(|_| AppError::Unauthorized("Invalid or expired session".to_string()))?;
    
    let user_id = token_data.claims.sub as i32;
    
    check_if_account_active(state, &user_id).await?;
    check_amount_being_transferred(state, &amount).await?;

    // to follow: paymongo integration for actual monetary transfer to central bank

    // create transaction record (standalone — no pending counterpart)
    let transaction = transactions_services::record_transaction(
        state,
        token,
        amount,
        Some(account_number.to_string()),
        None,
        "withdrawal".to_string(),
        "completed".to_string(),
        chrono::Utc::now().naive_utc(),
        None, // no pending counterpart, DB generates a fresh UUID
    )
    .await?;

    // todo!("Deposit functionality not implemented")
    Ok(WithdrawalResponse {
        message: "Withdrawal successful".to_string(),
        transaction: Some(transaction),
    })
}

pub async fn transfer(
    state: &AppState,
    token: &str,
    amount: Decimal,
    target_account_number: &str,
) -> Result<TransferResponse, AppError> {
    
    let token_data = decode::<JwtClaims>(
        token,
        &DecodingKey::from_secret(jwt_secret().as_ref()),
        &Validation::default(),
    )
    .map_err(|_| AppError::Unauthorized("Invalid or expired session".to_string()))?;
    
    let user_id = token_data.claims.sub as i32;
    let user_account_number = token_data.claims.account_number;
    
    check_if_account_active(state, &user_id).await?;
    check_amount_being_transferred(state, &amount).await?;

    // to follow: paymongo integration for actual monetary transfer to central bank

    // create transaction record (standalone — no pending counterpart)
    let transaction = transactions_services::record_transaction(
        state,
        token,
        amount,
        Some(user_account_number.to_string()),
        Some(target_account_number.to_string()),
        "transfer".to_string(),
        "completed".to_string(),
        chrono::Utc::now().naive_utc(),
        None, // no pending counterpart, DB generates a fresh UUID
    )
    .await?;

    // todo!("Deposit functionality not implemented")
    Ok(TransferResponse {
        message: "Transfer successful".to_string(),
        transaction: Some(transaction),
    })
}