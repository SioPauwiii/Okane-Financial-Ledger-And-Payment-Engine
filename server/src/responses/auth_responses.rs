use crate::models::users_models::User;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct RegisterResponse {
	pub message: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub user: Option<User>,
	pub access_token: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub message: String,
	pub access_token: String,
}

#[derive(Debug, Serialize)]
pub struct MeResponse {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub user: Option<User>,
}
