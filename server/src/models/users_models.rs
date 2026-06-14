use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
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
    pub birth_date: Option<NaiveDate>,
    pub contact_no: String,
    pub user_type: String, // customer, admin

    #[serde(skip_serializing)]
    pub password: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
} 