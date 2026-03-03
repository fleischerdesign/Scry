use serde::Serialize;
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
    pub title: String,
    pub photo_url: Option<String>,
    pub link: String,
}
