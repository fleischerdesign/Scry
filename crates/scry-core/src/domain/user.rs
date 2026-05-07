use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema, TS, Debug, Clone)]
#[ts(export)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub display_image: Option<String>,
}

#[derive(Clone, Debug, TS)]
#[ts(export)]
pub struct AuthContext {
    pub user_id: i64,
    #[allow(dead_code)]
    pub scopes: Vec<String>,
}
