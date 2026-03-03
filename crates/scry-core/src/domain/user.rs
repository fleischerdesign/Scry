use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct User {
    pub id: i64,
    pub username: String,
}

#[derive(Clone, Debug)]
pub struct AuthContext {
    pub user_id: i64,
    #[allow(dead_code)]
    pub scopes: Vec<String>,
}
