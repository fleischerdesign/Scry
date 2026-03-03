use axum::{
    http::Request,
    middleware::Next,
    response::Response,
    http::StatusCode,
};

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

/// Extracts identity from headers, query parameters, or peer IP.
pub async fn identity_resolver(
    mut req: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let identity = if let Some(key) = req.headers().get("X-API-Key").and_then(|h| h.to_str().ok()) {
        Identity::ApiKey(key.to_string())
    } else if let Some(query_key) = req.uri().query()
        .and_then(|q| serde_urlencoded::from_str::<std::collections::HashMap<String, String>>(q).ok())
        .and_then(|m| m.get("api_key").cloned()) {
        Identity::ApiKey(query_key)
    } else {
        let ip = req.extensions().get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
            .map(|ci| ci.0.ip().to_string())
            .unwrap_or_else(|| "unknown".to_string());
        Identity::Anonymous(ip)
    };

    req.extensions_mut().insert(identity);
    Ok(next.run(req).await)
}
