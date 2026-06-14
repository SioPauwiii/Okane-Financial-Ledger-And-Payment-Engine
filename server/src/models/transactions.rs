use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

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