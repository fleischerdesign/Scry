use sqlx::SqlitePool;
use crate::error::Result;

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
}
