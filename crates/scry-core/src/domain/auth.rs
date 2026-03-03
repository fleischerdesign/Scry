use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;
use crate::domain::user::User;
use ts_rs::TS;

#[derive(Deserialize, ToSchema, Validate, TS)]
#[ts(export)]
pub struct RegisterRequest {
    #[validate(length(min = 3, max = 50))]
    pub username: String,
    #[validate(length(min = 8, max = 100))]
    pub password: String,
}

#[derive(Deserialize, ToSchema, Validate, TS)]
#[ts(export)]
pub struct LoginRequest {
    #[validate(length(min = 3, max = 50))]
    pub username: String,
    pub password: String,
}

#[derive(Serialize, ToSchema, TS)]
#[ts(export)]
pub struct AuthResponse {
    pub api_key: String,
    pub user: User,
}
