use axum::{http::Request, http::StatusCode, middleware::Next, response::Response};

/// Represents the resolved identity of an incoming request.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Identity {
    ApiKey(String),
    Anonymous(String), // IP Address
}

impl Identity {
    pub fn as_str(&self) -> &str {
        match self {
            Identity::ApiKey(key) => key,
            Identity::Anonymous(ip) => ip,
        }
    }
}

/// Extracts identity from headers (Bearer/X-API-Key), query parameters, or peer IP.
pub async fn identity_resolver(
    mut req: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    // 1. Try Authorization: Bearer <token>
    let bearer_key = req
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(|s| s.to_string());

    // 2. Try X-API-Key header
    let api_key = req
        .headers()
        .get("X-API-Key")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    // 3. Try api_key in query string (for SSE)
    let query_key = req
        .uri()
        .query()
        .and_then(|q| {
            serde_urlencoded::from_str::<std::collections::HashMap<String, String>>(q).ok()
        })
        .and_then(|m| m.get("api_key").cloned());

    let identity = if let Some(key) = bearer_key.or(api_key).or(query_key) {
        Identity::ApiKey(key)
    } else {
        let ip = req
            .extensions()
            .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
            .map(|ci| ci.0.ip().to_string())
            .unwrap_or_else(|| "unknown".to_string());
        Identity::Anonymous(ip)
    };

    req.extensions_mut().insert(identity);
    Ok(next.run(req).await)
}
