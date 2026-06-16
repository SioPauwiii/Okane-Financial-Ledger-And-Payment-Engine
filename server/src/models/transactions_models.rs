use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Transaction {
    pub id: i32,
    pub from_account_number: Option<String>,
    pub to_account_number: Option<String>,
    pub amount_transferred: Decimal, // Using String to represent numeric values to avoid precision issues
    pub transaction_type: String, // transfer, deposit, withdrawal
    pub status: String, // pending, completed, failed
    pub created_at: NaiveDateTime,
}