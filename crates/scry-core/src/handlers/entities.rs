use axum::{
    extract::{State, Json, Path},
    Extension,
};
use std::sync::Arc;

use crate::domain::*;
use crate::error::Result;
use crate::state::AppState;

#[utoipa::path(get, path = "/api/v1/discovery/entities", responses((status = 200, body = [ApiNamespace])), security(("api_key" = [])))]
pub async fn get_namespaces(
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<AuthContext>
) -> Result<Json<Vec<ApiNamespace>>> {
    let res = state.graph_service.get_namespaces(auth.user_id).await?;
    Ok(Json(res))
}

#[utoipa::path(get, path = "/api/v1/discovery/entities/{namespace}", responses((status = 200, body = [ApiEntityType])), security(("api_key" = [])))]
pub async fn get_namespace_types(
    State(state): State<Arc<AppState>>,
    Path(namespace): Path<String>,
    Extension(auth): Extension<AuthContext>
) -> Result<Json<Vec<ApiEntityType>>> {
    let res = state.graph_service.get_namespace_types(auth.user_id, &namespace).await?;
    Ok(Json(res))
}

#[utoipa::path(get, path = "/api/v1/discovery/entities/{namespace}/{typ}", responses((status = 200, body = [ApiEntity])), security(("api_key" = [])))]
pub async fn get_entities(
    State(state): State<Arc<AppState>>,
    Path((namespace, typ)): Path<(String, String)>,
    Extension(auth): Extension<AuthContext>
) -> Result<Json<Vec<ApiEntity>>> {
    let res = state.graph_service.get_entities(auth.user_id, &namespace, &typ).await?;
    Ok(Json(res))
}

#[utoipa::path(get, path = "/api/v1/discovery/entities/{namespace}/{typ}/{id}/traits", responses((status = 200, body = serde_json::Value)), security(("api_key" = [])))]
pub async fn get_entity_traits(
    State(state): State<Arc<AppState>>,
    Path((namespace, typ, id)): Path<(String, String, String)>,
    Extension(auth): Extension<AuthContext>
) -> Result<Json<serde_json::Value>> {
    let res = state.graph_service.get_entity_details(auth.user_id, &namespace, &typ, &id).await?;
    Ok(Json(res))
}

#[utoipa::path(post, path = "/api/v1/discovery/resolve", responses((status = 200, body = [ApiEntity])), security(("api_key" = [])))]
pub async fn resolve_entities(
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<AuthContext>,
    Json(refs): Json<Vec<ApiEntityRef>>,
) -> Result<Json<Vec<ApiEntity>>> {
    let res = state.graph_service.resolve_entities(auth.user_id, refs).await?;
    Ok(Json(res))
}
