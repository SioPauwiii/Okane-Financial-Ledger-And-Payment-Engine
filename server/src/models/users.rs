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