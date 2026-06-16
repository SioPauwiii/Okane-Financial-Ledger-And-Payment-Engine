use serde::{Serialize, Deserialize};
use crate::models::transactions_models::Transaction;

#[derive(Debug, Serialize)]
pub struct TransactionResponse {
	pub message: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub transaction: Option<Transaction>,
}

#[derive(Debug, Serialize)]
pub struct GetTransactionsResponse {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub transactions: Option<Vec<Transaction>>,
}

#[derive(Debug, Serialize)]
pub struct MyTransactionsResponse {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub transactions: Option<Vec<Transaction>>,
}