use axum::{
    extract::{State, Json},
    response::IntoResponse,
    Extension,
};
use std::sync::Arc;
use validator::Validate;

use crate::domain::*;
use crate::error::Result;
use crate::state::AppState;

#[utoipa::path(post, path = "/api/v1/auth/register", request_body = RegisterRequest, responses((status = 200, body = AuthResponse)))]
pub async fn register_user(State(state): State<Arc<AppState>>, Json(req): Json<RegisterRequest>) -> Result<Json<AuthResponse>> {
    req.validate()?;
    let res = state.auth_service.register(req).await?;
    Ok(Json(res))
}

#[utoipa::path(post, path = "/api/v1/auth/login", request_body = LoginRequest, responses((status = 200, body = AuthResponse)))]
pub async fn login_user(State(state): State<Arc<AppState>>, Json(req): Json<LoginRequest>) -> Result<Json<AuthResponse>> {
    req.validate()?;
    let res = state.auth_service.login(req).await?;
    Ok(Json(res))
}

#[utoipa::path(get, path = "/api/v1/system/profile", responses((status = 200, body = serde_json::Value)), security(("api_key" = [])))]
pub async fn get_profile(State(state): State<Arc<AppState>>, Extension(auth): Extension<AuthContext>) -> Result<Json<serde_json::Value>> {
    let res = state.auth_service.get_profile(auth.user_id).await?;
    Ok(Json(res))
}

#[utoipa::path(post, path = "/api/v1/system/profile", responses((status = 200)), security(("api_key" = [])))]
pub async fn update_profile(State(state): State<Arc<AppState>>, Extension(auth): Extension<AuthContext>, Json(req): Json<serde_json::Map<String, serde_json::Value>>) -> Result<impl IntoResponse> {
    state.auth_service.update_profile(auth.user_id, req).await?;
    Ok(axum::http::StatusCode::OK)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;
    use crate::services::*;
    use crate::plugins::PluginManager;

    async fn setup_test_state() -> Arc<AppState> {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("../../migrations").run(&pool).await.unwrap();
        let pm = Arc::new(PluginManager::new("./non_existent", pool.clone()).unwrap());
        let svc = EventService::new(pool.clone(), pm.clone());
        let analytics = AnalyticsService::new(pool.clone(), pm.clone());
        let auth = AuthService::new(pool.clone());
        let dashboard = DashboardService::new(pool.clone());
        let graph = GraphService::new(pool.clone(), pm.clone());
        let secret = SecretService::new();
        let plugin = PluginService::new(pool.clone(), pm.clone(), svc.clone(), secret.clone());
        let system = SystemService::new(pool.clone());

        let (event_sender, _) = tokio::sync::broadcast::channel(1024);
        Arc::new(AppState { 
            db: pool.clone(),
            event_service: svc, 
            analytics_service: analytics,
            auth_service: auth,
            dashboard_service: dashboard,
            graph_service: graph,
            plugin_service: plugin,
            system_service: system,
            event_sender,
            cancel_token: tokio_util::sync::CancellationToken::new(),
            rate_limiter: Arc::new(crate::handlers::middleware::rate_limit::RateLimitState::new()),
        })
    }

    #[tokio::test]
    async fn test_auth_flow() {
        let state = setup_test_state().await;
        
        // 1. Zu kurzes Passwort (Validierung)
        let reg_fail = RegisterRequest { username: "alice".to_string(), password: "123".to_string() };
        let res = register_user(State(state.clone()), Json(reg_fail)).await;
        assert!(res.is_err()); // Sollte wegen Passwort-Länge (< 8) fehlschlagen

        // 2. Korrekte Registrierung
        let reg_ok = RegisterRequest { username: "alice".to_string(), password: "password123".to_string() };
        let res = register_user(State(state.clone()), Json(reg_ok)).await.unwrap();
        assert_eq!(res.user.username, "alice");
        let key = res.api_key.clone();
        assert!(!key.is_empty());

        // 3. Login
        let login = LoginRequest { username: "alice".to_string(), password: "password123".to_string() };
        let res_login = login_user(State(state.clone()), Json(login)).await.unwrap();
        assert_eq!(res_login.user.username, "alice");
        assert!(!res_login.api_key.is_empty());
        
        // 4. Wrong password
        let login_wrong = LoginRequest { username: "alice".to_string(), password: "wrong_password".to_string() };
        let res_wrong = login_user(State(state.clone()), Json(login_wrong)).await;
        assert!(res_wrong.is_err());
    }
}
