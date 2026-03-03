use axum::{
    extract::{State, Json},
    http::StatusCode,
    response::IntoResponse,
    Extension,
};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;
use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use password_hash::{PasswordHash, SaltString, rand_core::OsRng};
use validator::Validate;

use crate::models::*;
use crate::error::{Error, Result};

#[utoipa::path(post, path = "/api/v1/auth/register", request_body = RegisterRequest, responses((status = 200, body = AuthResponse)))]
pub async fn register_user(State(state): State<Arc<AppState>>, Json(req): Json<RegisterRequest>) -> Result<Json<AuthResponse>> {
    req.validate()?;
    let db = state.event_service.db();
    
    // Hash password with Argon2
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(req.password.as_bytes(), &salt)
        .map_err(|_| Error::Internal)?
        .to_string();

    let res = sqlx::query("INSERT INTO users (username, password_hash) VALUES (?, ?)")
        .bind(&req.username).bind(password_hash).execute(db).await?;
    
    let user_id = res.last_insert_rowid();
    
    // Ensure the 'self' user entity exists in the graph
    sqlx::query("INSERT INTO entities (user_id, namespace, typ, id) VALUES (?, ?, ?, ?) ON CONFLICT DO NOTHING")
        .bind(user_id).bind("scry.core").bind("user").bind("self").execute(db).await?;

    let api_key = Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO api_keys (key, user_id, label, scopes) VALUES (?, ?, ?, ?)")
        .bind(&api_key).bind(user_id).bind("Default Key").bind("all").execute(db).await?;

    Ok(Json(AuthResponse { api_key, user: User { id: user_id, username: req.username } }))
}

#[utoipa::path(post, path = "/api/v1/auth/login", request_body = LoginRequest, responses((status = 200, body = AuthResponse)))]
pub async fn login_user(State(state): State<Arc<AppState>>, Json(req): Json<LoginRequest>) -> Result<Json<AuthResponse>> {
    req.validate()?;
    let db = state.event_service.db();
    let user = sqlx::query_as::<_, (i64, String, String)>("SELECT id, username, password_hash FROM users WHERE username = ?")
        .bind(&req.username).fetch_optional(db).await?
        .ok_or_else(|| Error::Auth("User not found".to_string()))?;
    
    // Verify password with Argon2
    let parsed_hash = PasswordHash::new(&user.2)
        .map_err(|_| Error::Internal)?;
    
    if Argon2::default().verify_password(req.password.as_bytes(), &parsed_hash).is_err() {
        return Err(Error::Auth("Invalid password".to_string()));
    }

    // Ensure the 'self' user entity exists in the graph (migration for old users)
    sqlx::query("INSERT INTO entities (user_id, namespace, typ, id) VALUES (?, ?, ?, ?) ON CONFLICT DO NOTHING")
        .bind(user.0).bind("scry.core").bind("user").bind("self").execute(db).await?;

    let api_key = sqlx::query_scalar::<_, String>("SELECT key FROM api_keys WHERE user_id = ? LIMIT 1").bind(user.0).fetch_one(db).await?;
    Ok(Json(AuthResponse { api_key, user: User { id: user.0, username: user.1 } }))
}

#[utoipa::path(get, path = "/api/v1/system/profile", responses((status = 200, body = serde_json::Value)), security(("api_key" = [])))]
pub async fn get_profile(State(state): State<Arc<AppState>>, Extension(auth): Extension<AuthContext>) -> Result<Json<serde_json::Value>> {
    let db = state.event_service.db();

    // 1. Self-Healing: Ensure 'self' user entity exists
    sqlx::query("INSERT INTO entities (user_id, namespace, typ, id) VALUES (?, ?, ?, ?) ON CONFLICT DO NOTHING")
        .bind(auth.user_id).bind("scry.core").bind("user").bind("self").execute(db).await?;

    // 2. Load legacy profile rows
    let rows = sqlx::query_as::<_, (String, String)>("SELECT key, value FROM user_profile WHERE user_id = ?")
        .bind(auth.user_id).fetch_all(db).await?;
    
    let mut map = serde_json::Map::new();
    for (k, v) in rows {
        map.insert(k.clone(), json!(v));
        
        // 3. Auto-Sync to Knowledge Graph if not already there
        let trait_id = format!("scry.core/{}", k);
        let value_json = json!(v).to_string();
        
        sqlx::query("INSERT INTO entity_traits (user_id, namespace, entity_type, entity_id, plugin_id, trait_id, value_json) VALUES (?, ?, ?, ?, ?, ?, ?) ON CONFLICT DO NOTHING")
            .bind(auth.user_id).bind("scry.core").bind("user").bind("self").bind("core").bind(trait_id).bind(value_json).execute(db).await?;
    }
    
    Ok(Json(serde_json::Value::Object(map)))
}

#[utoipa::path(post, path = "/api/v1/system/profile", responses((status = 200)), security(("api_key" = [])))]
pub async fn update_profile(State(state): State<Arc<AppState>>, Extension(auth): Extension<AuthContext>, Json(req): Json<serde_json::Map<String, serde_json::Value>>) -> Result<impl IntoResponse> {
    let db = state.event_service.db();
    for (k, v) in req {
        let v_str = v.as_str().unwrap_or("").to_string();
        
        // 1. Update legacy table
        sqlx::query("INSERT INTO user_profile (user_id, key, value) VALUES (?, ?, ?) ON CONFLICT(user_id, key) DO UPDATE SET value = EXCLUDED.value")
            .bind(auth.user_id).bind(&k).bind(&v_str).execute(db).await?;

        // 2. Update semantic graph (Trait)
        // We use 'scry.core' as the trait namespace for profile values
        let trait_id = format!("scry.core/{}", k);
        let value_json = serde_json::to_string(&v).unwrap_or_else(|_| "null".to_string());
        
        sqlx::query("INSERT INTO entity_traits (user_id, namespace, entity_type, entity_id, plugin_id, trait_id, value_json) VALUES (?, ?, ?, ?, ?, ?, ?) ON CONFLICT(user_id, namespace, entity_type, entity_id, plugin_id, trait_id) DO UPDATE SET value_json = EXCLUDED.value_json")
            .bind(auth.user_id).bind("scry.core").bind("user").bind("self").bind("core").bind(trait_id).bind(value_json).execute(db).await?;
    }
    Ok(StatusCode::OK)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;
    use crate::event_service::EventService;
    use crate::analytics_service::AnalyticsService;
    use crate::plugins::PluginManager;

    async fn setup_test_state() -> Arc<AppState> {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("../../migrations").run(&pool).await.unwrap();
        let pm = Arc::new(PluginManager::new("./non_existent", pool.clone()).unwrap());
        let svc = EventService::new(pool.clone(), pm.clone());
        let analytics = AnalyticsService::new(pool, pm);
        let (event_sender, _) = tokio::sync::broadcast::channel(1024);
        Arc::new(AppState { 
            event_service: svc, 
            analytics_service: analytics,
            event_sender,
            cancel_token: tokio_util::sync::CancellationToken::new()
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

        // 3. Login
        let login = LoginRequest { username: "alice".to_string(), password: "password123".to_string() };
        let res_login = login_user(State(state.clone()), Json(login)).await.unwrap();
        assert_eq!(res_login.api_key, key);
        
        // 4. Falsches Passwort
        let login_wrong = LoginRequest { username: "alice".to_string(), password: "wrong_password".to_string() };
        let res_wrong = login_user(State(state.clone()), Json(login_wrong)).await;
        assert!(res_wrong.is_err());
    }
}
