use sqlx::SqlitePool;
use crate::error::Result;

pub struct EntityRepository<'a> {
    pool: &'a SqlitePool,
    user_id: i64,
}

impl<'a> EntityRepository<'a> {
    pub fn new(pool: &'a SqlitePool, user_id: i64) -> Self {
        Self { pool, user_id }
    }

    pub async fn ensure_entity(&self, namespace: &str, typ: &str, id: &str) -> Result<()> {
        sqlx::query("INSERT INTO entities (user_id, namespace, typ, id) VALUES (?, ?, ?, ?) ON CONFLICT DO NOTHING")
            .bind(self.user_id)
            .bind(namespace)
            .bind(typ)
            .bind(id)
            .execute(self.pool)
            .await?;
        Ok(())
    }

    pub async fn set_trait(&self, namespace: &str, entity_type: &str, entity_id: &str, plugin_id: &str, trait_id: &str, value_json: &str) -> Result<()> {
        sqlx::query("INSERT INTO entity_traits (user_id, namespace, entity_type, entity_id, plugin_id, trait_id, value_json) VALUES (?, ?, ?, ?, ?, ?, ?) ON CONFLICT(user_id, namespace, entity_type, entity_id, plugin_id, trait_id) DO UPDATE SET value_json = EXCLUDED.value_json")
            .bind(self.user_id)
            .bind(namespace)
            .bind(entity_type)
            .bind(entity_id)
            .bind(plugin_id)
            .bind(trait_id)
            .bind(value_json)
            .execute(self.pool)
            .await?;
        Ok(())
    }

    pub async fn set_trait_if_not_exists(&self, namespace: &str, entity_type: &str, entity_id: &str, plugin_id: &str, trait_id: &str, value_json: &str) -> Result<()> {
        sqlx::query("INSERT INTO entity_traits (user_id, namespace, entity_type, entity_id, plugin_id, trait_id, value_json) VALUES (?, ?, ?, ?, ?, ?, ?) ON CONFLICT DO NOTHING")
            .bind(self.user_id)
            .bind(namespace)
            .bind(entity_type)
            .bind(entity_id)
            .bind(plugin_id)
            .bind(trait_id)
            .bind(value_json)
            .execute(self.pool)
            .await?;
        Ok(())
    }
}
