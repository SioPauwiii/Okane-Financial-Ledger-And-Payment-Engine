use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: i64, // user id
    pub email: String,
    pub user_type: String, // customer, admin
    pub exp: usize, // expiration time as a timestamp
}