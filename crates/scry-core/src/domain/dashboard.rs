use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use ts_rs::TS;

#[derive(Serialize, Deserialize, ToSchema, TS, Debug, Clone)]
#[ts(export)]
pub struct Dashboard {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub is_default: bool,
    pub widgets: Vec<DashboardWidget>,
}

#[derive(Serialize, Deserialize, ToSchema, TS, Debug, Clone)]
#[ts(export)]
pub struct DashboardWidget {
    pub id: String,
    pub dashboard_id: String,
    pub r#type: String,
    pub title: Option<String>,
    pub config: serde_json::Value,
    pub width_span: i32,
    pub sort_order: i32,
}
