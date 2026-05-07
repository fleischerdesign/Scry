use axum::{Json, extract::Query, response::IntoResponse};
use serde::Deserialize;
use utoipa::IntoParams;

use crate::domain::{SemanticMapping, SemanticResolver};

#[derive(Deserialize, IntoParams)]
pub struct ResolveQuery {
    #[serde(rename = "type")]
    pub scry_type: String,
}

#[utoipa::path(
    get,
    path = "/api/v1/semantic/resolve",
    params(ResolveQuery),
    responses((status = 200, body = SemanticMapping, description = "Returns the schema.org mapping for a given Scry semantic type")),
    security(("api_key" = []))
)]
pub async fn resolve_semantic_type(Query(query): Query<ResolveQuery>) -> impl IntoResponse {
    let mapping = SemanticResolver::get_mapping(&query.scry_type);
    Json(mapping)
}
