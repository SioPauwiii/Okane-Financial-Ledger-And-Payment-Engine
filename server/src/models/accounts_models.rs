use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

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