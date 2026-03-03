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

    pub async fn get_namespaces(&self) -> Result<Vec<String>> {
        let names = sqlx::query_scalar::<_, String>("SELECT DISTINCT namespace FROM entities WHERE user_id = ?")
            .bind(self.user_id)
            .fetch_all(self.pool)
            .await?;
        Ok(names)
    }

    pub async fn get_types_by_namespace(&self, namespace: &str) -> Result<Vec<String>> {
        let types = sqlx::query_scalar::<_, String>("SELECT DISTINCT typ FROM entities WHERE user_id = ? AND namespace = ?")
            .bind(self.user_id)
            .bind(namespace)
            .fetch_all(self.pool)
            .await?;
        Ok(types)
    }

    pub async fn get_entities_by_type(&self, namespace: &str, typ: &str) -> Result<Vec<(String, Option<String>, Option<String>)>> {
        let rows = sqlx::query_as::<_, (String, Option<String>, Option<String>)>("
            SELECT e.id, (
                SELECT value_json FROM entity_traits t 
                WHERE t.user_id = e.user_id AND t.namespace = e.namespace AND t.entity_type = e.typ AND t.entity_id = e.id 
                AND (t.trait_id LIKE '%name' OR t.trait_id LIKE '%title')
                LIMIT 1
            ) as title,
            (
                SELECT value_json FROM entity_traits t 
                WHERE t.user_id = e.user_id AND t.namespace = e.namespace AND t.entity_type = e.typ AND t.entity_id = e.id 
                AND (t.trait_id LIKE '%photo' OR t.trait_id LIKE '%avatar' OR t.trait_id LIKE '%image')
                LIMIT 1
            ) as photo
            FROM entities e
            WHERE e.user_id = ? AND e.namespace = ? AND e.typ = ?
        ")
        .bind(self.user_id)
        .bind(namespace)
        .bind(typ)
        .fetch_all(self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn get_traits(&self, namespace: &str, typ: &str, id: &str) -> Result<Vec<(String, String, String)>> {
        let rows = sqlx::query_as::<_, (String, String, String)>("SELECT plugin_id, trait_id, value_json FROM entity_traits WHERE user_id = ? AND namespace = ? AND entity_type = ? AND entity_id = ?")
            .bind(self.user_id)
            .bind(namespace)
            .bind(typ)
            .bind(id)
            .fetch_all(self.pool)
            .await?;
        Ok(rows)
    }

    pub async fn get_relationships(&self, namespace: &str, typ: &str, id: &str) -> Result<Vec<(String, String, String, String, String, String, String)>> {
        let rel_rows = sqlx::query_as::<_, (String, String, String, String, String, String, String)>("SELECT source_ns, source_type, source_id, predicate, target_ns, target_type, target_id FROM entity_relationships WHERE user_id = ? AND (source_ns = ? AND source_type = ? AND source_id = ? OR target_ns = ? AND target_type = ? AND target_id = ?)")
            .bind(self.user_id)
            .bind(namespace).bind(typ).bind(id)
            .bind(namespace).bind(typ).bind(id)
            .fetch_all(self.pool)
            .await?;
        Ok(rel_rows)
    }
}
