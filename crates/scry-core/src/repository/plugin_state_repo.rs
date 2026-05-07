use crate::error::Result;
use crate::plugins::scry::plugin::host::{QueryParam, Relationship};
use crate::services::SecretService;
use sqlx::SqlitePool;

pub struct PluginStateRepository {
    pool: SqlitePool,
    user_id: i64,
    plugin_name: String,
    secret_service: SecretService,
}

impl PluginStateRepository {
    pub fn new(pool: &SqlitePool, user_id: i64, plugin_name: &str) -> Self {
        Self {
            pool: pool.clone(),
            user_id,
            plugin_name: plugin_name.to_string(),
            secret_service: SecretService::new(),
        }
    }

    pub async fn set_state(&self, key: &str, value: &str) -> Result<()> {
        sqlx::query("INSERT INTO plugin_state (user_id, plugin_name, key, value) VALUES (?, ?, ?, ?) ON CONFLICT(user_id, plugin_name, key) DO UPDATE SET value = EXCLUDED.value, updated_at = CURRENT_TIMESTAMP")
            .bind(self.user_id).bind(&self.plugin_name).bind(key).bind(value).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn get_state(&self, key: &str) -> Result<Option<String>> {
        let row = sqlx::query_scalar::<_, String>(
            "SELECT value FROM plugin_state WHERE user_id = ? AND plugin_name = ? AND key = ?",
        )
        .bind(self.user_id)
        .bind(&self.plugin_name)
        .bind(key)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn get_config(&self, key: &str) -> Result<Option<String>> {
        let res = sqlx::query_scalar::<_, String>("SELECT value FROM plugin_config WHERE user_id = ? AND plugin_id = ? AND key = ? AND is_secret = 0")
            .bind(self.user_id).bind(&self.plugin_name).bind(key).fetch_optional(&self.pool).await?;
        Ok(res)
    }

    pub async fn get_secret(&self, key: &str) -> Result<Option<String>> {
        let res = sqlx::query_scalar::<_, String>("SELECT value FROM plugin_config WHERE user_id = ? AND plugin_id = ? AND key = ? AND is_secret = 1")
            .bind(self.user_id).bind(&self.plugin_name).bind(key).fetch_optional(&self.pool).await?;

        match res {
            Some(encrypted) => Ok(self.secret_service.decrypt(self.user_id, &encrypted)?),
            None => Ok(None),
        }
    }

    pub async fn set_trait(
        &self,
        namespace: &str,
        typ: &str,
        id: &str,
        trait_id: &str,
        value_json: &str,
    ) -> Result<()> {
        sqlx::query("INSERT INTO entities (user_id, namespace, typ, id) VALUES (?, ?, ?, ?) ON CONFLICT DO NOTHING")
            .bind(self.user_id).bind(namespace).bind(typ).bind(id).execute(&self.pool).await?;

        sqlx::query("INSERT INTO entity_traits (user_id, plugin_id, namespace, entity_type, entity_id, trait_id, value_json) VALUES (?, ?, ?, ?, ?, ?, ?) ON CONFLICT(user_id, plugin_id, namespace, entity_type, entity_id, trait_id) DO UPDATE SET value_json = EXCLUDED.value_json, updated_at = CURRENT_TIMESTAMP")
            .bind(self.user_id).bind(&self.plugin_name).bind(namespace).bind(typ).bind(id).bind(trait_id).bind(value_json).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn get_trait(
        &self,
        namespace: &str,
        typ: &str,
        id: &str,
        trait_id: &str,
    ) -> Result<Option<String>> {
        let row = sqlx::query_scalar::<_, String>(
            "SELECT value_json FROM entity_traits WHERE user_id = ? AND namespace = ? AND entity_type = ? AND entity_id = ? AND trait_id = ? ORDER BY updated_at DESC LIMIT 1"
        )
        .bind(self.user_id).bind(namespace).bind(typ).bind(id).bind(trait_id)
        .fetch_optional(&self.pool).await?;
        Ok(row)
    }

    pub async fn set_relationship(&self, rel: Relationship) -> Result<()> {
        sqlx::query("INSERT INTO entity_relationships (user_id, plugin_id, source_ns, source_type, source_id, predicate, target_ns, target_type, target_id) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?) ON CONFLICT DO UPDATE SET updated_at = CURRENT_TIMESTAMP")
            .bind(self.user_id)
            .bind(&self.plugin_name)
            .bind(rel.source_namespace)
            .bind(rel.source_type)
            .bind(rel.source_id)
            .bind(rel.predicate)
            .bind(rel.target_namespace)
            .bind(rel.target_type)
            .bind(rel.target_id)
            .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn get_relationships(
        &self,
        namespace: &str,
        typ: &str,
        id: &str,
        direction: &str,
    ) -> Result<Vec<Relationship>> {
        let sql = if direction == "in" {
            "SELECT source_ns, source_type, source_id, predicate, target_ns, target_type, target_id FROM entity_relationships WHERE user_id = ? AND target_ns = ? AND target_type = ? AND target_id = ?"
        } else {
            "SELECT source_ns, source_type, source_id, predicate, target_ns, target_type, target_id FROM entity_relationships WHERE user_id = ? AND source_ns = ? AND source_type = ? AND source_id = ?"
        };

        let rows =
            sqlx::query_as::<_, (String, String, String, String, String, String, String)>(sql)
                .bind(self.user_id)
                .bind(namespace)
                .bind(typ)
                .bind(id)
                .fetch_all(&self.pool)
                .await?;

        Ok(rows
            .into_iter()
            .map(|(sn, st, si, p, tn, tt, ti)| Relationship {
                source_namespace: sn,
                source_type: st,
                source_id: si,
                predicate: p,
                target_namespace: tn,
                target_type: tt,
                target_id: ti,
            })
            .collect())
    }

    pub async fn raw_query(
        &self,
        sql: &str,
        user_id: i64,
        params: Vec<QueryParam>,
    ) -> Result<Vec<sqlx::sqlite::SqliteRow>> {
        let safe_sql = format!(
            "WITH events AS (SELECT id, user_id, timestamp, category, source, json(payload) as payload, metadata FROM main.events WHERE user_id = ?) {}",
            sql
        );
        let mut query = sqlx::query(&safe_sql).bind(user_id);

        for param in params {
            query = match param {
                QueryParam::S(s) => query.bind(s),
                QueryParam::I(i) => query.bind(i),
                QueryParam::F(f) => query.bind(f),
            };
        }

        let rows = query.fetch_all(&self.pool).await?;
        Ok(rows)
    }
}
