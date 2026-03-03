use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct ApiNamespace {
    pub name: String,
    pub icon: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct ApiEntity {
    pub namespace: String,
    pub typ: String,
    pub id: String,
    pub title: String,
    pub photo_url: Option<String>,
    pub link: String,
}
