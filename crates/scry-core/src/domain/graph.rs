use serde::{Serialize, Deserialize};
use utoipa::ToSchema;
use ts_rs::TS;

#[derive(Serialize, ToSchema, TS)]
#[ts(export)]
pub struct ApiNamespace {
    pub name: String,
    pub icon: Option<String>,
}

#[derive(Serialize, ToSchema, TS)]
#[ts(export)]
pub struct ApiEntity {
    pub namespace: String,
    pub typ: String,
    pub id: String,
    pub display_title: String,
    pub display_image: Option<String>,
}

#[derive(Serialize, Deserialize, ToSchema, TS)]
#[ts(export)]
pub struct ApiEntityRef {
    pub namespace: String,
    pub typ: String,
    pub id: String,
}
