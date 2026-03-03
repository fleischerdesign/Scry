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
}
