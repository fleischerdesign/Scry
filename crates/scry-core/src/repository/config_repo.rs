use sqlx::SqlitePool;
use crate::error::Result;

pub struct ConfigRepository<'a> {
    pool: &'a SqlitePool,
    user_id: i64,
}

impl<'a> ConfigRepository<'a> {
    pub fn new(pool: &'a SqlitePool, user_id: i64) -> Self {
        Self { pool, user_id }
    }

    pub async fn get(&self, plugin_id: &str, key: &str) -> Result<Option<String>> {
        let val = sqlx::query_scalar::<_, String>(
            "SELECT value FROM plugin_config WHERE user_id = ? AND plugin_id = ? AND key = ?"
        )
        .bind(self.user_id).bind(plugin_id).bind(key).fetch_optional(self.pool).await?;
        Ok(val)
    }

    pub async fn get_all_by_plugin(&self, plugin_id: &str) -> Result<Vec<(String, String)>> {
        let rows = sqlx::query_as::<_, (String, String)>(
            "SELECT key, value FROM plugin_config WHERE user_id = ? AND plugin_id = ?"
        )
        .bind(self.user_id).bind(plugin_id).fetch_all(self.pool).await?;
        Ok(rows)
    }

    pub async fn set(&self, plugin_id: &str, key: &str, value: &str) -> Result<()> {
        sqlx::query("INSERT INTO plugin_config (user_id, plugin_id, key, value) VALUES (?, ?, ?, ?) ON CONFLICT(user_id, plugin_id, key) DO UPDATE SET value = EXCLUDED.value")
            .bind(self.user_id).bind(plugin_id).bind(key).bind(value).execute(self.pool).await?;
        Ok(())
    }

    pub async fn set_if_not_exists(&self, plugin_id: &str, key: &str, value: &str) -> Result<()> {
        sqlx::query("INSERT INTO plugin_config (user_id, plugin_id, key, value) VALUES (?, ?, ?, ?) ON CONFLICT DO NOTHING")
            .bind(self.user_id).bind(plugin_id).bind(key).bind(value).execute(self.pool).await?;
        Ok(())
    }
}
