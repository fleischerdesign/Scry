use sqlx::SqlitePool;
use crate::error::Result;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
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
        loop {
            tokio::select! {
                _ = token.cancelled() => {
                    tracing::info!("Background scheduler shutting down...");
                    break;
                }
                _ = async {
                    let user_ids = match state.auth_service.get_all_user_ids().await {
                        Ok(ids) => ids,
                        Err(e) => {
                            tracing::error!("Failed to fetch users for scheduler: {}", e);
                            vec![]
                        }
                    };

                    let manifests = state.event_service.plugin_manager().get_plugin_manifests().await;
                    
                    for user_id in user_ids {
                        for (name, _) in &manifests {
                            let svc = state.event_service.clone();
                            let plugin_name = name.clone();
                            tokio::spawn(async move {
                                if let Err(e) = svc.poll_and_save_plugin(user_id, &plugin_name).await {
                                    tracing::warn!(user_id = %user_id, plugin = %plugin_name, "Scheduler poll failed: {}", e);
                                }
                            });
                        }
                    }
                    sleep(Duration::from_secs(60)).await;
                } => {}
            }
        }
    }
}
