use crate::domain::AuthContext;
use crate::handlers::middleware::identity::Identity;
use crate::state::AppState;
use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    mut req: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    if req.method() == axum::http::Method::OPTIONS {
        return Ok(next.run(req).await);
    }

    let identity = req
        .extensions()
        .get::<Identity>()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    match identity {
        Identity::ApiKey(key) => {
            // Dual Verification:
            // 1. Try if it's a JWT (fast, no DB call)
            let auth = if let Some(jwt_auth) = state.auth_service.verify_jwt(key) {
                Some(jwt_auth)
            } else {
                // 2. Fallback to classic API Key in DB
                state.auth_service.verify_api_key(key).await.map_err(|e| {
                    tracing::error!("Auth Service Error: {}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?
            };

            if let Some((user_id, scopes)) = auth {
                let ctx = AuthContext { user_id, scopes };
                req.extensions_mut().insert(ctx);
                Ok(next.run(req).await)
            } else {
                tracing::warn!("Invalid API Key or JWT");
                Err(StatusCode::UNAUTHORIZED)
            }
        }
        Identity::Anonymous(_) => Err(StatusCode::UNAUTHORIZED),
    }
}
