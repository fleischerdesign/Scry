use sqlx::SqlitePool;
use crate::error::Result;
use serde_json::Value;

pub struct AnalyticsRepository<'a> {
    pool: &'a SqlitePool,
    user_id: i64,
}

impl<'a> AnalyticsRepository<'a> {
    pub fn new(pool: &'a SqlitePool, user_id: i64) -> Self {
        Self { pool, user_id }
    }

    pub async fn correlate_nearest(&self, base_category: &str, join_category: &str, limit: u32) -> Result<Vec<Value>> {
        let sql = r#"
            SELECT 
                CAST(b.payload AS TEXT),
                CAST(j.payload AS TEXT),
                b.entities,
                b.display_title,
                b.display_subtitle
            FROM events b
            JOIN events j ON j.category = ? AND j.user_id = ?
            WHERE b.category = ? AND b.user_id = ?
            GROUP BY b.id
            HAVING MIN(ABS(julianday(substr(b.timestamp, 1, 19)) - julianday(substr(j.timestamp, 1, 19))))
            ORDER BY b.timestamp DESC
            LIMIT ?
        "#;

        let rows = sqlx::query_as::<_, (String, Option<String>, Option<String>, Option<String>, Option<String>)>(sql)
            .bind(join_category).bind(self.user_id)
            .bind(base_category).bind(self.user_id)
            .bind(limit)
            .fetch_all(self.pool).await?;

        Ok(rows.into_iter().map(|(b, j, e, dt, ds)| {
            serde_json::json!({
                "base": serde_json::from_str::<Value>(&b).unwrap_or_default(),
                "joined": j.and_then(|s| serde_json::from_str::<Value>(&s).ok()).unwrap_or_default(),
                "entities": e.and_then(|s| serde_json::from_str::<Value>(&s).ok()).unwrap_or_default(),
                "display_title": dt,
                "display_subtitle": ds,
            })
        }).collect())
    }

    pub async fn get_semantic_top(&self, category: &str, path: &str, limit: u32, days: Option<u32>) -> Result<Vec<Value>> {
        let mut sql = format!(
            "SELECT payload ->> '{}' as key, COUNT(*) as count FROM events WHERE user_id = ? AND category = ?",
            path
        );

        if days.is_some() {
            sql.push_str(" AND timestamp > date('now', ?)");
        }

        sql.push_str(" GROUP BY key ORDER BY count DESC LIMIT ?");

        let mut query = sqlx::query_as::<_, (Option<String>, i64)>(&sql)
            .bind(self.user_id).bind(category);

        if let Some(d) = days {
            query = query.bind(format!("-{} days", d));
        }

        let rows = query.bind(limit).fetch_all(self.pool).await?;

        Ok(rows.into_iter().map(|(k, c)| {
            serde_json::json!({ "key": k.unwrap_or_else(|| "Unknown".to_string()), "count": c })
        }).collect())
    }

    pub async fn get_semantic_series(&self, category: &str, path: &str, days: u32, interval: Option<String>) -> Result<Vec<Value>> {
        let format_str = match interval.as_deref() {
            Some("1h") => "%Y-%m-%dT%H:00:00Z",
            _ => "%Y-%m-%d",
        };

        let sql = format!(
            "SELECT strftime('{}', timestamp) as label, AVG(CAST(payload ->> '{}' as REAL)) as value FROM events WHERE user_id = ? AND category = ? AND timestamp > date('now', ?) GROUP BY label ORDER BY label ASC",
            format_str, path
        );

        let rows = sqlx::query_as::<_, (String, f64)>(&sql)
            .bind(self.user_id)
            .bind(category)
            .bind(format!("-{} days", days))
            .fetch_all(self.pool).await?;

        Ok(rows.into_iter().map(|(l, v)| {
            serde_json::json!({ "label": l, "value": v })
        }).collect())
    }
}
