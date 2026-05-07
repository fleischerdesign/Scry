use crate::error::Result;
use crate::services::SecretService;
use sqlx::SqlitePool;

pub struct ConfigRepository<'a> {
    pool: &'a SqlitePool,
    user_id: i64,
    secret_service: &'a SecretService,
}

impl<'a> ConfigRepository<'a> {
    pub fn new(pool: &'a SqlitePool, user_id: i64, secret_service: &'a SecretService) -> Self {
        Self {
            pool,
            user_id,
            secret_service,
        }
    }

    pub async fn get(&self, plugin_id: &str, key: &str) -> Result<Option<String>> {
        let val = sqlx::query_scalar::<_, String>(
            "SELECT value FROM plugin_config WHERE user_id = ? AND plugin_id = ? AND key = ? AND is_secret = 0"
        )
        .bind(self.user_id).bind(plugin_id).bind(key).fetch_optional(self.pool).await?;
        Ok(val)
    }

    pub async fn get_all_by_plugin(&self, plugin_id: &str) -> Result<Vec<(String, String, bool)>> {
        let rows = sqlx::query_as::<_, (String, String, bool)>(
            "SELECT key, value, is_secret FROM plugin_config WHERE user_id = ? AND plugin_id = ?",
        )
        .bind(self.user_id)
        .bind(plugin_id)
        .fetch_all(self.pool)
        .await?;

        let mut result = Vec::new();
        for (key, value, is_secret) in rows {
            if is_secret {
                if let Ok(Some(decrypted)) = self.secret_service.decrypt(self.user_id, &value) {
                    result.push((key, decrypted, is_secret));
                }
            } else {
                result.push((key, value, is_secret));
            }
        }
        Ok(result)
    }

    pub async fn get_secrets_by_plugin(&self, plugin_id: &str) -> Result<Vec<(String, String)>> {
        let rows = sqlx::query_as::<_, (String, String)>(
            "SELECT key, value FROM plugin_config WHERE user_id = ? AND plugin_id = ? AND is_secret = 1"
        )
        .bind(self.user_id).bind(plugin_id).fetch_all(self.pool).await?;

        let mut result = Vec::new();
        for (key, encrypted_value) in rows {
            if let Ok(Some(decrypted)) = self.secret_service.decrypt(self.user_id, &encrypted_value)
            {
                result.push((key, decrypted));
            }
        }
        Ok(result)
    }

    pub async fn set(
        &self,
        plugin_id: &str,
        key: &str,
        value: &str,
        is_secret: bool,
    ) -> Result<()> {
        let value_to_store = if is_secret {
            self.secret_service.encrypt(self.user_id, value)?
        } else {
            value.to_string()
        };

        sqlx::query("INSERT INTO plugin_config (user_id, plugin_id, key, value, is_secret) VALUES (?, ?, ?, ?, ?) ON CONFLICT(user_id, plugin_id, key) DO UPDATE SET value = EXCLUDED.value, is_secret = EXCLUDED.is_secret")
            .bind(self.user_id).bind(plugin_id).bind(key).bind(value_to_store).bind(is_secret).execute(self.pool).await?;
        Ok(())
    }

    pub async fn set_if_not_exists(
        &self,
        plugin_id: &str,
        key: &str,
        value: &str,
        is_secret: bool,
    ) -> Result<()> {
        let value_to_store = if is_secret {
            self.secret_service.encrypt(self.user_id, value)?
        } else {
            value.to_string()
        };

        sqlx::query("INSERT INTO plugin_config (user_id, plugin_id, key, value, is_secret) VALUES (?, ?, ?, ?, ?) ON CONFLICT DO NOTHING")
            .bind(self.user_id).bind(plugin_id).bind(key).bind(value_to_store).bind(is_secret).execute(self.pool).await?;
        Ok(())
    }

    pub async fn get_secret(&self, plugin_id: &str, key: &str) -> Result<Option<String>> {
        let val = sqlx::query_scalar::<_, String>(
            "SELECT value FROM plugin_config WHERE user_id = ? AND plugin_id = ? AND key = ? AND is_secret = 1"
        )
        .bind(self.user_id).bind(plugin_id).bind(key).fetch_optional(self.pool).await?;

        match val {
            Some(encrypted) => Ok(self.secret_service.decrypt(self.user_id, &encrypted)?),
            None => Ok(None),
        }
    }
}
