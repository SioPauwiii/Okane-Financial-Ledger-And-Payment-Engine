use serde::{Deserialize, Serialize};
use chrono::NaiveDate;
use crate::models::users_models::User;

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub confirm_password: String,
    pub first_name: String,
    pub last_name: String,
    pub country: Option<String>,
    pub city: Option<String>,
    pub street: String,
    pub house_no: String,
    pub zip_code: String,
    pub contact_no: String,
    pub sex: Option<String>,
    pub birth_date: Option<NaiveDate>,
    pub nationality: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub message: String,
    pub user: User,
    pub access_token: Option<String>,
}