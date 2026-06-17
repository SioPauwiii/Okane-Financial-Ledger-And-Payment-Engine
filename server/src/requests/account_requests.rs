use serde::Deserialize;
use chrono::NaiveDate;
use rust_decimal::Decimal;

#[derive(Debug, Deserialize)]
pub struct DepositRequest {
    pub amount: Decimal,
    pub account_number: String,
}

#[derive(Debug, Deserialize)]
pub struct WithdrawalRequest {
    pub amount: Decimal,
    pub account_number: String,
}

#[derive(Debug, Deserialize)]
pub struct TransferRequest {
    pub amount: Decimal,
    pub target_account_number: String,
}