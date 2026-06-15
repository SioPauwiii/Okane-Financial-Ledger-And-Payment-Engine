use crate::models::users_models::User;
use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize)]
pub struct MeResponse {
    pub user: User,
}