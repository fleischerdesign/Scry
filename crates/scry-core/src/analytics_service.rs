use std::sync::Arc;
use sqlx::SqlitePool;
use crate::plugins::PluginManager;
use crate::error::Result;
use serde_json::{Value, json};

#[derive(Clone)]
pub struct AnalyticsService {
    db: SqlitePool,
    plugin_manager: Arc<PluginManager>,
}

impl AnalyticsService {
    pub fn new(db: SqlitePool, plugin_manager: Arc<PluginManager>) -> Self {
        Self { db, plugin_manager }
    }

    /// Findet Korrelationen zwischen allen numerischen semantic_types eines Benutzers.
    pub async fn run_correlation_discovery(&self, user_id: i64) -> Result<usize> {
        let manifests = self.plugin_manager.get_plugin_manifests().await;
        
        // 1. Alle numerischen semantic_types sammeln
        let mut numeric_targets = Vec::new();
        for m in manifests.values() {
            for export in &m.exports {
                // Wir nehmen an, dass Typen wie 'temperature', 'bpm', 'level', 'count' numerisch sind.
                // In einer finalen Version könnten wir 'format' im WIT nutzen.
                let is_numeric = export.semantic_type.contains("temp") || 
                                export.semantic_type.contains("bpm") || 
                                export.semantic_type.contains("level") ||
                                export.semantic_type.contains("humidity");
                
                if is_numeric {
                    numeric_targets.push((export.category.clone(), export.path.clone(), export.semantic_type.clone()));
                }
            }
        }

        if numeric_targets.len() < 2 { return Ok(0); }

        let mut discoveries = 0;

        // 2. Pairwise correlation (O(n^2), aber n ist klein bei Plugins)
        for i in 0..numeric_targets.len() {
            for j in i + 1..numeric_targets.len() {
                let (cat_a, path_a, sem_a) = &numeric_targets[i];
                let (cat_b, path_b, sem_b) = &numeric_targets[j];

                if let Some(corr) = self.calculate_pearson(user_id, cat_a, path_a, cat_b, path_b).await? {
                    if corr.abs() > 0.6 { // Signifikante Korrelation
                        self.store_correlation(user_id, sem_a, sem_b, corr).await?;
                        discoveries += 1;
                    }
                }
            }
        }

        Ok(discoveries)
    }

    async fn calculate_pearson(&self, user_id: i64, cat_a: &str, path_a: &str, cat_b: &str, path_b: &str) -> Result<Option<f64>> {
        let path_a_clean = path_a.strip_prefix("payload.").unwrap_or(path_a);
        let path_b_clean = path_b.strip_prefix("payload.").unwrap_or(path_b);

        // SQL: Wir gruppieren in 1-Stunden-Buckets und nehmen den Durchschnitt
        let sql = format!(r#"
            WITH series_a AS (
                SELECT strftime('%Y-%m-%d %H:00:00', timestamp) as bucket,
                       AVG(CAST(payload ->> '{}' as REAL)) as val
                FROM events WHERE user_id = ? AND category = ?
                GROUP BY bucket
            ),
            series_b AS (
                SELECT strftime('%Y-%m-%d %H:00:00', timestamp) as bucket,
                       AVG(CAST(payload ->> '{}' as REAL)) as val
                FROM events WHERE user_id = ? AND category = ?
                GROUP BY bucket
            )
            SELECT a.val as val_a, b.val as val_b
            FROM series_a a
            JOIN series_b b ON a.bucket = b.bucket
        "#, path_a_clean, path_b_clean);

        let rows = sqlx::query_as::<_, (f64, f64)>(&sql)
            .bind(user_id).bind(cat_a)
            .bind(user_id).bind(cat_b)
            .fetch_all(&self.db).await?;

        if rows.len() < 5 { return Ok(None); } // Zu wenige Datenpunkte

        let n = rows.len() as f64;
        let (mut sum_a, mut sum_b, mut sum_ab, mut sum_a2, mut sum_b2) = (0.0, 0.0, 0.0, 0.0, 0.0);

        for (a, b) in rows {
            sum_a += a;
            sum_b += b;
            sum_ab += a * b;
            sum_a2 += a * a;
            sum_b2 += b * b;
        }

        let num = n * sum_ab - sum_a * sum_b;
        let den = ((n * sum_a2 - sum_a * sum_a) * (n * sum_b2 - sum_b * sum_b)).sqrt();

        if den == 0.0 { Ok(None) } else { Ok(Some(num / den)) }
    }

    async fn store_correlation(&self, user_id: i64, sem_a: &str, sem_b: &str, strength: f64) -> Result<()> {
        let predicate = "scry.core/correlates_with";
        let metadata = json!({ "strength": strength, "method": "pearson_1h_buckets" });
        let metadata_str = serde_json::to_string(&metadata).unwrap();

        // Wir speichern die Korrelation als Beziehung zwischen den semantischen Typen (virtuelle Entitäten)
        sqlx::query(r#"
            INSERT INTO entity_relationships (user_id, plugin_id, source_ns, source_type, source_id, predicate, target_ns, target_type, target_id, metadata)
            VALUES (?, 'core', ?, 'semantic_type', ?, ?, ?, 'semantic_type', ?, ?)
            ON CONFLICT(user_id, plugin_id, source_ns, source_type, source_id, predicate, target_ns, target_type, target_id) 
            DO UPDATE SET metadata = EXCLUDED.metadata
        "#)
        .bind(user_id)
        .bind("scry.core").bind(sem_a)
        .bind(predicate)
        .bind("scry.core").bind(sem_b)
        .bind(metadata_str)
        .execute(&self.db).await?;

        Ok(())
    }

    pub async fn get_discoveries(&self, user_id: i64) -> Result<Vec<Value>> {
        let rows = sqlx::query_as::<_, (String, String, String)>(
            "SELECT source_id, target_id, metadata FROM entity_relationships 
             WHERE user_id = ? AND predicate = 'scry.core/correlates_with'
             ORDER BY json_extract(metadata, '$.strength') DESC"
        )
        .bind(user_id).fetch_all(&self.db).await?;

        Ok(rows.into_iter().map(|(s, t, m)| {
            json!({
                "source": s,
                "target": t,
                "insights": m,
                "display_text": format!("Correlation found between {} and {}", s, t)
            })
        }).collect())
    }
}
