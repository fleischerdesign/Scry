use std::sync::Arc;
use sqlx::SqlitePool;
use crate::plugins::PluginManager;
use crate::error::Result;
use crate::repository::AnalyticsRepository;
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

    pub async fn run_correlation_discovery(&self, user_id: i64) -> Result<usize> {
        let manifests = self.plugin_manager.get_plugin_manifests().await;
        let mut numeric_targets = Vec::new();
        for m in manifests.values() {
            for export in &m.exports {
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
        let repo = AnalyticsRepository::new(&self.db, user_id);

        for i in 0..numeric_targets.len() {
            for j in i + 1..numeric_targets.len() {
                let (cat_a, path_a, sem_a) = &numeric_targets[i];
                let (cat_b, path_b, sem_b) = &numeric_targets[j];

                let path_a_clean = path_a.strip_prefix("payload.").unwrap_or(path_a);
                let path_b_clean = path_b.strip_prefix("payload.").unwrap_or(path_b);

                let rows = repo.calculate_pearson_series(cat_a, path_a_clean, cat_b, path_b_clean).await?;

                if rows.len() >= 5 {
                    if let Some(corr) = self.calculate_pearson_coefficient(rows) {
                        if corr.abs() > 0.6 {
                            let metadata = json!({ "strength": corr, "method": "pearson_1h_buckets" });
                            repo.store_discovery(sem_a, sem_b, &serde_json::to_string(&metadata).unwrap()).await?;
                            discoveries += 1;
                        }
                    }
                }
            }
        }

        Ok(discoveries)
    }

    fn calculate_pearson_coefficient(&self, rows: Vec<(f64, f64)>) -> Option<f64> {
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

        if den == 0.0 { None } else { Some(num / den) }
    }

    pub async fn get_discoveries(&self, user_id: i64) -> Result<Vec<Value>> {
        let repo = AnalyticsRepository::new(&self.db, user_id);
        let rows = repo.get_discoveries().await?;

        Ok(rows.into_iter().map(|(s, t, m)| {
            json!({
                "source": s,
                "target": t,
                "insights": m,
                "display_text": format!("Correlation found between {} and {}", s, t)
            })
        }).collect())
    }

    pub async fn search(&self, user_id: i64, query: &str, limit: u32) -> Result<Vec<Value>> {
        let repo = AnalyticsRepository::new(&self.db, user_id);
        let rows = repo.search(query, limit).await?;

        let results: Vec<Value> = rows.into_iter().map(|(id, typ, snippet, subtext, link)| {
            json!({
                "id": id,
                "type": typ,
                "label": if typ == "event" { subtext.clone() } else { id.clone() },
                "snippet": snippet,
                "subtext": subtext,
                "link": link
            })
        }).collect();

        Ok(results)
    }
}
