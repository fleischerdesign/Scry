use crate::error::Result;
use sqlx::SqlitePool;

pub struct ProfileRepository<'a> {
    pool: &'a SqlitePool,
    user_id: i64,
}

impl<'a> ProfileRepository<'a> {
    pub fn new(pool: &'a SqlitePool, user_id: i64) -> Self {
        Self { pool, user_id }
    }

    pub async fn get_all(&self) -> Result<Vec<(String, String)>> {
        let rows = sqlx::query_as::<_, (String, String)>(
            "SELECT key, value FROM user_profile WHERE user_id = ?",
        )
        .bind(self.user_id)
        .fetch_all(self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn set(&self, key: &str, value: &str) -> Result<()> {
        sqlx::query("INSERT INTO user_profile (user_id, key, value) VALUES (?, ?, ?) ON CONFLICT(user_id, key) DO UPDATE SET value = EXCLUDED.value")
            .bind(self.user_id)
            .bind(key)
            .bind(value)
            .execute(self.pool)
            .await?;
        Ok(())
    }
}
