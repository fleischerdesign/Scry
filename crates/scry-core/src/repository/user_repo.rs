use sqlx::SqlitePool;
use crate::error::Result;

pub struct UserRepository<'a> {
    pool: &'a SqlitePool,
}

impl<'a> UserRepository<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create_user(&self, username: &str, password_hash: &str) -> Result<i64> {
        let res = sqlx::query("INSERT INTO users (username, password_hash) VALUES (?, ?)")
            .bind(username)
            .bind(password_hash)
            .execute(self.pool)
            .await?;
        Ok(res.last_insert_rowid())
    }

    pub async fn find_by_username(&self, username: &str) -> Result<Option<(i64, String, String)>> {
        let user = sqlx::query_as::<_, (i64, String, String)>("SELECT id, username, password_hash FROM users WHERE username = ?")
            .bind(username)
            .fetch_optional(self.pool)
            .await?;
        Ok(user)
    }

    pub async fn create_api_key(&self, user_id: i64, key: &str, label: &str, scopes: &str) -> Result<()> {
        sqlx::query("INSERT INTO api_keys (key, user_id, label, scopes) VALUES (?, ?, ?, ?)")
            .bind(key)
            .bind(user_id)
            .bind(label)
            .bind(scopes)
            .execute(self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_api_key_by_user(&self, user_id: i64) -> Result<String> {
        let key = sqlx::query_scalar::<_, String>("SELECT key FROM api_keys WHERE user_id = ? LIMIT 1")
            .bind(user_id)
            .fetch_one(self.pool)
            .await?;
        Ok(key)
    }

    pub async fn get_all_ids(&self) -> Result<Vec<i64>> {
        let ids = sqlx::query_scalar::<_, i64>("SELECT id FROM users")
            .fetch_all(self.pool)
            .await?;
        Ok(ids)
    }

    pub async fn find_by_api_key(&self, key: &str) -> Result<Option<(i64, String)>> {
        let auth = sqlx::query_as::<_, (i64, String)>("SELECT user_id, scopes FROM api_keys WHERE key = ?")
            .bind(key)
            .fetch_optional(self.pool)
            .await?;
        Ok(auth)
    }
}
