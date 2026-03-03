use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{Response, IntoResponse},
};
use std::sync::Arc;
use crate::state::AppState;
use crate::handlers::middleware::identity::Identity;
use governor::{Quota, RateLimiter, state::keyed::DashMapStateStore};
use std::num::NonZeroU32;
use serde_json::json;

type KeyedRateLimiter = RateLimiter<String, DashMapStateStore<String>, governor::clock::DefaultClock>;

pub struct RateLimitState {
    limiter: KeyedRateLimiter,
}

impl RateLimitState {
    pub fn new() -> Self {
        let quota = Quota::per_second(NonZeroU32::new(25).unwrap())
            .allow_burst(NonZeroU32::new(50).unwrap());
        
        Self {
            limiter: RateLimiter::dashmap(quota),
        }
    }

    pub fn check(&self, identity: &Identity) -> bool {
        self.limiter.check_key(&identity.as_str().to_string()).is_ok()
    }

    pub fn cleanup(&self) {
        self.limiter.retain_recent();
    }
}

pub async fn rate_limit_middleware(
    State(state): State<Arc<AppState>>,
    req: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let identity = req.extensions().get::<Identity>()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    if state.rate_limiter.check(identity) {
        Ok(next.run(req).await)
    } else {
        tracing::warn!("Rate limit exceeded for identity: {:?}", identity);
        Ok((
            StatusCode::TOO_MANY_REQUESTS,
            axum::Json(json!({
                "error": "Rate limit exceeded",
                "detail": "Too many requests. Please slow down."
            })),
        ).into_response())
    }
}
