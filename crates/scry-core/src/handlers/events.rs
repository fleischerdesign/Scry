use axum::{
    extract::{State, Json, Query, Path},
    response::sse::{Event as SseEvent, Sse},
    Extension,
};
use scry_proto::Event;
use std::sync::Arc;
use futures::stream::Stream;
use std::convert::Infallible;

use crate::domain::*;
use crate::error::{Error, Result};
use crate::state::AppState;

pub async fn stream_live_events(
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<AuthContext>,
) -> Sse<impl Stream<Item = std::result::Result<SseEvent, Infallible>>> {
    let mut rx = state.event_sender.subscribe();
    let cancel_token = state.cancel_token.clone();

    let stream = async_stream::stream! {
        loop {
            tokio::select! {
                // Beende den Stream, wenn der Server herunterfährt
                _ = cancel_token.cancelled() => {
                    break;
                }
                // Warte auf neue Events
                res = rx.recv() => {
                    match res {
                        Ok(event) => {
                            let is_user_event = event.metadata.as_ref()
                                .and_then(|m| m.get("user_id"))
                                .and_then(|u| u.as_i64()) == Some(auth.user_id);
                            
                            if is_user_event {
                                if let Ok(data) = serde_json::to_string(&event) {
                                    yield Ok(SseEvent::default().data(data));
                                }
                            }
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                        Err(_) => break,
                    }
                }
            }
        }
    };

    Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::default())
}

#[utoipa::path(get, path = "/api/v1/data/{path}", responses((status = 200, body = [Event])), security(("api_key" = [])))]
pub async fn get_data_by_type(State(state): State<Arc<AppState>>, Path(path): Path<String>, Query(params): Query<ListParams>, Extension(auth): Extension<AuthContext>) -> Result<Json<Vec<Event>>> {
    let semantic_path = path.replace('/', ".");
    let events = state.event_service.search_semantic(auth.user_id, &semantic_path, params.limit.unwrap_or(100), params.offset.unwrap_or(0)).await?;
    Ok(Json(events))
}

#[utoipa::path(get, path = "/api/v1/streams/timeline", params(ListParams), responses((status = 200, body = [serde_json::Value])), security(("api_key" = [])))]
pub async fn get_timeline(State(state): State<Arc<AppState>>, Query(params): Query<ListParams>, Extension(auth): Extension<AuthContext>) -> Result<Json<Vec<serde_json::Value>>> {
    let timeline = state.event_service.get_enriched_timeline(auth.user_id, params.category, params.limit.unwrap_or(20), params.offset.unwrap_or(0)).await?;
    Ok(Json(timeline))
}

#[utoipa::path(get, path = "/api/v1/streams/summary", params(SummaryParams), responses((status = 200, body = [String])), security(("api_key" = [])))]
pub async fn get_daily_summary(State(state): State<Arc<AppState>>, Query(params): Query<SummaryParams>, Extension(auth): Extension<AuthContext>) -> Result<Json<Vec<String>>> {
    let date = params.date.as_deref().unwrap_or("2026-02-28");
    let summary = state.event_service.generate_daily_summary(auth.user_id, date).await?;
    Ok(Json(summary))
}

#[utoipa::path(get, path = "/api/v1/data/id/{id}", responses((status = 200, body = serde_json::Value)), security(("api_key" = [])))]
pub async fn get_event_by_id(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Extension(auth): Extension<AuthContext>
) -> Result<Json<serde_json::Value>> {
    let ev: Option<Event> = state.event_service.get_event_by_id(auth.user_id, &id).await?;
    let ev = ev.ok_or_else(|| Error::NotFound(format!("Event {} not found", id)))?;
    
    Ok(Json(serde_json::to_value(ev).unwrap()))
}

#[utoipa::path(post, path = "/api/v1/ingest", request_body = Event, responses((status = 200, body = Event)), security(("api_key" = [])))]
pub async fn ingest_event(State(state): State<Arc<AppState>>, Extension(auth): Extension<AuthContext>, Json(event): Json<Event>) -> Result<Json<Event>> {
    let event = state.event_service.ingest_event(auth.user_id, event).await?;
    Ok(Json(event))
}

#[utoipa::path(get, path = "/api/v1/data/entity/{namespace}/{typ}/{id}", responses((status = 200, body = Vec<serde_json::Value>)), security(("api_key" = [])))]
pub async fn get_events_by_entity(
    State(state): State<Arc<AppState>>,
    Path((namespace, typ, id)): Path<(String, String, String)>,
    Extension(auth): Extension<AuthContext>
) -> Result<Json<serde_json::Value>> {
    let events: Vec<Event> = state.event_service.get_events_by_entity(auth.user_id, &namespace, &typ, &id).await?;
    
    let json_events: Vec<serde_json::Value> = events.into_iter()
        .map(|e| serde_json::to_value(e).unwrap())
        .collect();

    Ok(Json(serde_json::Value::Array(json_events)))
}
