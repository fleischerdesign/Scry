use sqlx::SqlitePool;
use crate::error::Result;
use std::sync::Arc;
use tokio::time::Duration;
use tokio_util::sync::CancellationToken;

#[derive(Clone)]
pub struct SystemService {
    db: SqlitePool,
}

impl SystemService {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }

    pub async fn health_check(&self) -> Result<()> {
        self.db.acquire().await.map_err(|e| crate::error::Error::Database(e))?;
        Ok(())
    }

    pub async fn get_status(&self) -> serde_json::Value {
        serde_json::json!({ "status": "online", "multi_tenant": true })
    }

    pub async fn run_background_tasks(&self, state: Arc<crate::state::AppState>, token: CancellationToken) {
        tracing::info!("Starting background scheduler...");
        
        // Use a shorter tick interval to check more frequently for due plugins
        let mut interval = tokio::time::interval(Duration::from_secs(10));
        
        loop {
            tokio::select! {
                _ = token.cancelled() => {
                    tracing::info!("Background scheduler shutting down...");
                    break;
                }
                _ = interval.tick() => {
                    let user_ids = match state.auth_service.get_all_user_ids().await {
                        Ok(ids) => ids,
                        Err(e) => {
                            tracing::error!("Failed to fetch users for scheduler: {}", e);
                            vec![]
                        }
                    };

                    let manifests = state.event_service.plugin_manager().get_plugin_manifests().await;
                    let now = chrono::Utc::now();

                    for user_id in user_ids {
                        for (plugin_name, manifest) in &manifests {
                            // Only poll if interval is defined
                            let Some(poll_secs) = manifest.poll_interval else { continue; };
                            
                            let repo = crate::repository::PluginStateRepository::new(&self.db, user_id, plugin_name);
                            
                            // Check last poll time from state
                            let should_poll = match repo.get_state("__system:last_poll").await {
                                Ok(Some(last_poll_str)) => {
                                    if let Ok(last_poll) = chrono::DateTime::parse_from_rfc3339(&last_poll_str) {
                                        now.signed_duration_since(last_poll.with_timezone(&chrono::Utc)) >= chrono::Duration::seconds(poll_secs as i64)
                                    } else { true }
                                },
                                Ok(None) => true,
                                Err(_) => false,
                            };

                            if should_poll {
                                let svc = state.event_service.clone();
                                let p_name = plugin_name.clone();
                                let p_repo = crate::repository::PluginStateRepository::new(&self.db, user_id, &p_name);
                                
                                tokio::spawn(async move {
                                    tracing::debug!(user_id = %user_id, plugin = %p_name, "Scheduled poll starting...");
                                    // Update last poll first to avoid double triggers if polling takes a while
                                    let _ = p_repo.set_state("__system:last_poll", &chrono::Utc::now().to_rfc3339()).await;
                                    
                                    if let Err(e) = svc.poll_and_save_plugin(user_id, &p_name).await {
                                        tracing::warn!(user_id = %user_id, plugin = %p_name, "Scheduled poll failed: {}", e);
                                    }
                                });
                            }
                        }
                    }
                }
            }
        }
    }
}
