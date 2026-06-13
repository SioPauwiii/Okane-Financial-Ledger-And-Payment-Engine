use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
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

    #[serde(skip_serializing)]
    pub password: String,
    pub user_type: String, // customer, admin
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    pub id: i64,
    pub user_id: i64,
    pub account_number: String,
    pub account_type: String, // checking, savings
    pub currency: String,
    pub balance: Decimal, // Using Decimal to represent numeric values to avoid precision issues
    pub status: String, // active, closed, frozen
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Transaction {
    pub id: i64,
    pub from_account_id: i64,
    pub to_account_id: i64,
    pub amount_transferred: Decimal, // Using String to represent numeric values to avoid precision issues
    pub transaction_type: String, // transfer, deposit, withdrawal
    pub status: String, // pending, completed, failed
    pub created_at: NaiveDateTime,
}