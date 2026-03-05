use axum::{
    extract::{State, Json, Path, Query},
    http::{StatusCode, header},
    response::{IntoResponse, Redirect},
    Extension,
};
use std::sync::Arc;

use crate::domain::*;
use crate::error::Result;
use crate::state::AppState;
use serde::Deserialize;
use base64::Engine;
use crate::services::SecretService;

#[utoipa::path(get, path = "/api/v1/system/plugins/{id}/reports/{report_id}", responses((status = 200, body = ApiReportData)), security(("api_key" = [])))]
pub async fn run_plugin_report(
    State(state): State<Arc<AppState>>,
    Path((id, report_id)): Path<(String, String)>,
    Extension(auth): Extension<AuthContext>
) -> Result<Json<ApiReportData>> {
    let res = state.plugin_service.run_plugin_report(auth.user_id, &id, report_id).await?;
    Ok(Json(res))
}

#[utoipa::path(get, path = "/api/v1/system/plugins/{id}/config", responses((status = 200, body = serde_json::Value)), security(("api_key" = [])))]
pub async fn get_plugin_config(State(state): State<Arc<AppState>>, Path(id): Path<String>, Extension(auth): Extension<AuthContext>) -> Result<Json<serde_json::Value>> {
    let res = state.plugin_service.get_plugin_config(auth.user_id, &id).await?;
    Ok(Json(res))
}

#[utoipa::path(get, path = "/api/v1/system/plugins/{id}/secrets", responses((status = 200, body = serde_json::Value)), security(("api_key" = [])))]
pub async fn get_plugin_secrets(State(state): State<Arc<AppState>>, Path(id): Path<String>, Extension(auth): Extension<AuthContext>) -> Result<Json<serde_json::Value>> {
    let res = state.plugin_service.get_plugin_secrets(auth.user_id, &id).await?;
    Ok(Json(res))
}

#[utoipa::path(post, path = "/api/v1/system/plugins/{id}/config", responses((status = 200)), security(("api_key" = [])))]
pub async fn update_plugin_config(State(state): State<Arc<AppState>>, Path(id): Path<String>, Extension(auth): Extension<AuthContext>, Json(req): Json<serde_json::Map<String, serde_json::Value>>) -> Result<impl IntoResponse> {
    state.plugin_service.update_plugin_config(auth.user_id, &id, req).await?;
    Ok(StatusCode::OK)
}

#[utoipa::path(get, path = "/api/v1/discovery/catalog", responses((status = 200, description = "Catalog")), security(("api_key" = [])))]
pub async fn get_catalog(State(state): State<Arc<AppState>>, Extension(_auth): Extension<AuthContext>) -> impl IntoResponse {
    let catalog = state.plugin_service.get_catalog().await;
    Json(catalog)
}

#[utoipa::path(get, path = "/api/v1/system/plugins", responses((status = 200, body = [PluginStatus])), security(("api_key" = [])))]
pub async fn get_system_plugins(State(state): State<Arc<AppState>>, Extension(auth): Extension<AuthContext>) -> Result<Json<Vec<PluginStatus>>> {
    let res = state.plugin_service.get_system_plugins(auth.user_id).await?;
    Ok(Json(res))
}

#[utoipa::path(post, path = "/api/v1/system/plugins/{id}/poll", responses((status = 200, description = "Poll")), security(("api_key" = [])))]
pub async fn poll_plugin_manually(State(state): State<Arc<AppState>>, Path(id): Path<String>, Extension(auth): Extension<AuthContext>) -> Result<Json<serde_json::Value>> {
    let count = state.plugin_service.poll_plugin_manually(auth.user_id, &id).await?;
    Ok(Json(serde_json::json!({ "plugin": id, "events_saved": count })))
}

#[derive(Deserialize)]
pub struct SpotifyOAuthQuery {
    code: Option<String>,
    error: Option<String>,
    state: Option<String>,
}

#[utoipa::path(get, path = "/api/v1/system/plugins/{id}/auth-url", responses((status = 200, body = serde_json::Value)), security(("api_key" = [])))]
pub async fn get_plugin_auth_url(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Extension(auth): Extension<AuthContext>,
) -> Result<Json<serde_json::Value>> {
    match id.as_str() {
        "scry-spotify-plugin" => {
            let client_id = std::env::var("SPOTIFY_CLIENT_ID")
                .unwrap_or_else(|_| "".to_string());
            
            if client_id.is_empty() {
                return Ok(Json(serde_json::json!({ "error": "SPOTIFY_CLIENT_ID not configured" })));
            }
            
            let redirect_uri = std::env::var("SPOTIFY_REDIRECT_URI")
                .unwrap_or_else(|_| "http://127.0.0.1:3000/api/v1/system/plugins/spotify/callback".to_string());
            
            let scopes = [
                "user-read-recently-played",
                "user-read-currently-playing",
                "user-read-playback-state",
            ].join(" ");
            
            let state = format!("user_{}", auth.user_id);
            let auth_url = format!(
                "https://accounts.spotify.com/authorize?response_type=code&client_id={}&redirect_uri={}&scope={}&state={}",
                urlencoding::encode(&client_id),
                urlencoding::encode(&redirect_uri),
                urlencoding::encode(&scopes),
                urlencoding::encode(&state)
            );
            
            Ok(Json(serde_json::json!({ "auth_url": auth_url, "state": state })))
        }
        _ => Ok(Json(serde_json::json!({ "error": "Plugin does not support OAuth" })))
    }
}

#[utoipa::path(get, path = "/api/v1/system/plugins/spotify/callback", responses((status = 200, description = "OAuth callback")))]
pub async fn spotify_oauth_callback(
    State(app_state): State<Arc<AppState>>,
    Query(query): Query<SpotifyOAuthQuery>,
) -> impl IntoResponse {
    if let Some(error) = query.error {
        return Redirect::to(&format!("/settings?error=spotify_auth_failed&message={}", urlencoding::encode(&error)));
    }

    let code = match query.code {
        Some(c) => c,
        None => {
            return Redirect::to("/settings?error=spotify_no_code");
        }
    };

    let oauth_state = match query.state {
        Some(s) => s,
        None => return Redirect::to("/settings?error=spotify_no_state"),
    };

    let user_id: i64 = match oauth_state.strip_prefix("user_") {
        Some(id_str) => id_str.parse().unwrap_or(0),
        None => return Redirect::to("/settings?error=spotify_invalid_state"),
    };

    if user_id == 0 {
        return Redirect::to("/settings?error=spotify_invalid_user");
    }

    let client_id = std::env::var("SPOTIFY_CLIENT_ID")
        .unwrap_or_else(|_| "".to_string());
    let client_secret = std::env::var("SPOTIFY_CLIENT_SECRET")
        .unwrap_or_else(|_| "".to_string());
    let redirect_uri = std::env::var("SPOTIFY_REDIRECT_URI")
        .unwrap_or_else(|_| "http://127.0.0.1:3000/api/v1/system/plugins/spotify/callback".to_string());

    if client_id.is_empty() || client_secret.is_empty() {
        return Redirect::to("/settings?error=spotify_not_configured");
    }

    let credentials = base64::engine::general_purpose::STANDARD.encode(
        format!("{}:{}", client_id, client_secret)
    );

    let params = [
        ("grant_type", "authorization_code"),
        ("code", &code),
        ("redirect_uri", &redirect_uri),
    ];

    let client = reqwest::Client::new();
    let response = client
        .post("https://accounts.spotify.com/api/token")
        .form(&params)
        .header("Authorization", format!("Basic {}", credentials))
        .send()
        .await;

    match response {
        Ok(resp) if resp.status().is_success() => {
            let token: serde_json::Value = resp.json().await.unwrap_or_default();
            let refresh_token = token.get("refresh_token").and_then(|v| v.as_str()).unwrap_or("");
            
            if !refresh_token.is_empty() {
                let secret_service = SecretService::new();
                let repo = crate::repository::ConfigRepository::new(&app_state.db, user_id, &secret_service);
                if let Err(e) = repo.set("scry-spotify-plugin", "refresh_token", refresh_token, true).await {
                    tracing::error!("Failed to save Spotify refresh token: {}", e);
                    return Redirect::to("/settings?error=spotify_save_failed");
                }
            }
            
            Redirect::to("/settings?spotify_connected=true")
        }
        _ => {
            Redirect::to("/settings?error=spotify_tokenExchange_failed")
        }
    }
}
