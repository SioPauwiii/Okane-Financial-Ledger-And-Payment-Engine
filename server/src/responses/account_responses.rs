use crate::{
    models::users_models::User, 
    models::accounts_models::Account
};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct UserAccountDetails {
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub country: Option<String>,
    pub province: Option<String>,
    pub city: Option<String>,
    pub street: String,
    pub house_no: String,
    pub zip_code: String,
    pub sex: Option<String>,
    pub nationality: Option<String>,
    pub birth_date: Option<NaiveDate>,
    pub contact_no: String,
    pub user_type: String,

    pub account_number: String,
    pub account_type: String,
    pub currency: String,
    pub balance: Decimal,
    pub status: String,
}

#[derive(Serialize, Deserialize)]
pub struct MyAccountResponse {
    pub user_account: Option<UserAccountDetails>,
}

#[derive(sqlx::FromRow)]
pub struct UserWithAccountRow {
    #[sqlx(flatten)]
    pub user: User,
    pub account: Option<sqlx::types::Json<Account>>,
}