use axum::{
    Extension,
    extract::{Json, Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
};
use std::sync::Arc;

use crate::domain::*;
use crate::error::Result;
use crate::services::SecretService;
use crate::state::AppState;
use base64::Engine;
use serde::Deserialize;

#[utoipa::path(get, path = "/api/v1/system/plugins/{id}/reports/{report_id}", responses((status = 200, body = ApiReportData)), security(("api_key" = [])))]
pub async fn run_plugin_report(
    State(state): State<Arc<AppState>>,
    Path((id, report_id)): Path<(String, String)>,
    Extension(auth): Extension<AuthContext>,
) -> Result<Json<ApiReportData>> {
    let res = state
        .plugin_service
        .run_plugin_report(auth.user_id, &id, report_id)
        .await?;
    Ok(Json(res))
}

#[utoipa::path(get, path = "/api/v1/system/plugins/{id}/config", responses((status = 200, body = serde_json::Value)), security(("api_key" = [])))]
pub async fn get_plugin_config(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Extension(auth): Extension<AuthContext>,
) -> Result<Json<serde_json::Value>> {
    let res = state
        .plugin_service
        .get_plugin_config(auth.user_id, &id)
        .await?;
    Ok(Json(res))
}

#[utoipa::path(get, path = "/api/v1/system/plugins/{id}/secrets", responses((status = 200, body = serde_json::Value)), security(("api_key" = [])))]
pub async fn get_plugin_secrets(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Extension(auth): Extension<AuthContext>,
) -> Result<Json<serde_json::Value>> {
    let res = state
        .plugin_service
        .get_plugin_secrets(auth.user_id, &id)
        .await?;
    Ok(Json(res))
}

#[utoipa::path(post, path = "/api/v1/system/plugins/{id}/config", responses((status = 200)), security(("api_key" = [])))]
pub async fn update_plugin_config(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Extension(auth): Extension<AuthContext>,
    Json(req): Json<serde_json::Map<String, serde_json::Value>>,
) -> Result<impl IntoResponse> {
    state
        .plugin_service
        .update_plugin_config(auth.user_id, &id, req)
        .await?;
    Ok(StatusCode::OK)
}

#[utoipa::path(get, path = "/api/v1/discovery/catalog", responses((status = 200, description = "Catalog")), security(("api_key" = [])))]
pub async fn get_catalog(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthContext>,
) -> impl IntoResponse {
    let catalog = state.plugin_service.get_catalog().await;
    Json(catalog)
}

#[utoipa::path(get, path = "/api/v1/system/plugins", responses((status = 200, body = [PluginStatus])), security(("api_key" = [])))]
pub async fn get_system_plugins(
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<AuthContext>,
) -> Result<Json<Vec<PluginStatus>>> {
    let res = state
        .plugin_service
        .get_system_plugins(auth.user_id)
        .await?;
    Ok(Json(res))
}

#[utoipa::path(post, path = "/api/v1/system/plugins/{id}/poll", responses((status = 200, description = "Poll")), security(("api_key" = [])))]
pub async fn poll_plugin_manually(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Extension(auth): Extension<AuthContext>,
) -> Result<Json<serde_json::Value>> {
    let count = state
        .plugin_service
        .poll_plugin_manually(auth.user_id, &id)
        .await?;
    Ok(Json(
        serde_json::json!({ "plugin": id, "events_saved": count }),
    ))
}

#[derive(Deserialize)]
pub struct OAuthQuery {
    code: Option<String>,
    error: Option<String>,
    state: Option<String>,
}

#[utoipa::path(get, path = "/api/v1/system/plugins/{id}/auth", responses((status = 200, body = serde_json::Value)), security(("api_key" = [])))]
pub async fn plugin_oauth_start(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Extension(auth): Extension<AuthContext>,
) -> Result<Json<serde_json::Value>> {
    let oauth_config = state.plugin_service.get_oauth_config(&id).await?;

    let (client_id, _client_secret) = state
        .plugin_service
        .get_oauth_credentials(auth.user_id, &id)
        .await?;

    let redirect_uri = format!(
        "http://127.0.0.1:3000/api/v1/system/plugins/{}/auth/callback",
        id
    );

    let scopes = oauth_config.scopes.join(" ");
    let state_val = format!("user_{}", auth.user_id);

    let auth_url = format!(
        "{}?response_type=code&client_id={}&redirect_uri={}&scope={}&state={}",
        oauth_config.auth_url,
        urlencoding::encode(&client_id),
        urlencoding::encode(&redirect_uri),
        urlencoding::encode(&scopes),
        urlencoding::encode(&state_val)
    );

    Ok(Json(
        serde_json::json!({ "auth_url": auth_url, "state": state_val }),
    ))
}

#[utoipa::path(get, path = "/api/v1/system/plugins/{id}/auth/callback", responses((status = 200, description = "OAuth callback")))]
pub async fn plugin_oauth_callback(
    State(app_state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Query(query): Query<OAuthQuery>,
) -> impl IntoResponse {
    if let Some(error) = query.error {
        return Redirect::to(&format!(
            "/settings?error=oauth_failed&message={}",
            urlencoding::encode(&error)
        ));
    }

    let code = match query.code {
        Some(c) => c,
        None => return Redirect::to("/settings?error=oauth_no_code"),
    };

    let oauth_state = match query.state {
        Some(s) => s,
        None => return Redirect::to("/settings?error=oauth_no_state"),
    };

    let user_id: i64 = match oauth_state.strip_prefix("user_") {
        Some(id_str) => id_str.parse().unwrap_or(0),
        None => return Redirect::to("/settings?error=oauth_invalid_state"),
    };

    if user_id == 0 {
        return Redirect::to("/settings?error=oauth_invalid_user");
    }

    let (client_id, client_secret) = match app_state
        .plugin_service
        .get_oauth_credentials(user_id, &id)
        .await
    {
        Ok(creds) => creds,
        Err(_) => return Redirect::to("/settings?error=oauth_no_credentials"),
    };

    let oauth_config = match app_state.plugin_service.get_oauth_config(&id).await {
        Ok(config) => config,
        Err(_) => return Redirect::to("/settings?error=oauth_not_supported"),
    };

    let redirect_uri = format!(
        "http://127.0.0.1:3000/api/v1/system/plugins/{}/auth/callback",
        id
    );

    let credentials = base64::engine::general_purpose::STANDARD
        .encode(format!("{}:{}", client_id, client_secret));

    let params = [
        ("grant_type", "authorization_code"),
        ("code", &code),
        ("redirect_uri", &redirect_uri),
    ];

    tracing::info!(plugin_id = %id, user_id = %user_id, "Exchanging OAuth code for token at {}", oauth_config.token_url);

    let client = reqwest::Client::new();
    let response = client
        .post(&oauth_config.token_url)
        .form(&params)
        .header("Authorization", format!("Basic {}", credentials))
        .header("Accept", "application/json")
        .send()
        .await;

    match response {
        Ok(resp) => {
            let status = resp.status();
            let body_text = resp.text().await.unwrap_or_default();

            tracing::info!(plugin_id = %id, status = %status.as_u16(), "GitHub responded to token exchange");

            if status.is_success() {
                let token: serde_json::Value = serde_json::from_str(&body_text).unwrap_or_default();
                let access_token = token
                    .get("access_token")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let refresh_token = token
                    .get("refresh_token")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                if access_token.is_empty() {
                    tracing::warn!(plugin_id = %id, body = %body_text, "Successful HTTP response but access_token is empty or missing in JSON");
                }

                let secret_service = SecretService::new();
                let repo = crate::repository::ConfigRepository::new(
                    &app_state.db,
                    user_id,
                    &secret_service,
                );

                if !access_token.is_empty() {
                    if let Err(e) = repo
                        .set(&id, "oauth_access_token", access_token, true)
                        .await
                    {
                        tracing::error!(plugin_id = %id, error = %e, "Failed to save oauth access token to database");
                    } else {
                        tracing::info!(plugin_id = %id, "OAuth access token saved successfully");
                    }
                }

                if !refresh_token.is_empty()
                    && let Err(e) = repo
                        .set(&id, "oauth_refresh_token", refresh_token, true)
                        .await
                {
                    tracing::error!(plugin_id = %id, error = %e, "Failed to save oauth refresh token to database");
                }

                Redirect::to("/settings?oauth_connected=true")
            } else {
                tracing::error!(plugin_id = %id, status = %status.as_u16(), body = %body_text, "GitHub token exchange failed with non-success status");
                Redirect::to("/settings?error=oauth_token_failed")
            }
        }
        Err(e) => {
            tracing::error!(plugin_id = %id, error = %e, "HTTP error during GitHub token exchange");
            Redirect::to("/settings?error=oauth_token_failed")
        }
    }
}
