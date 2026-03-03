use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use ts_rs::TS;

#[derive(Serialize, Deserialize, ToSchema, TS, Debug, Clone)]
#[ts(export)]
pub struct User {
    pub id: i64,
    pub username: String,
}

#[derive(Clone, Debug, TS)]
#[ts(export)]
pub struct AuthContext {
    pub user_id: i64,
    #[allow(dead_code)]
    pub scopes: Vec<String>,
}
