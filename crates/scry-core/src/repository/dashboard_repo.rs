use crate::domain::{Dashboard, DashboardWidget};
use crate::error::Result;
use serde_json::json;
use sqlx::SqlitePool;

pub struct DashboardRepository<'a> {
    pool: &'a SqlitePool,
    user_id: i64,
}

impl<'a> DashboardRepository<'a> {
    pub fn new(pool: &'a SqlitePool, user_id: i64) -> Self {
        Self { pool, user_id }
    }

    pub async fn create(&self, id: &str, name: &str, slug: &str) -> Result<()> {
        sqlx::query("INSERT INTO dashboards (id, user_id, name, slug) VALUES (?, ?, ?, ?)")
            .bind(id)
            .bind(self.user_id)
            .bind(name)
            .bind(slug)
            .execute(self.pool)
            .await?;
        Ok(())
    }

    pub async fn list(&self) -> Result<Vec<Dashboard>> {
        let dashboards = sqlx::query_as::<_, (String, String, String, bool)>("SELECT id, name, COALESCE(slug, id) as slug, is_default FROM dashboards WHERE user_id = ?")
            .bind(self.user_id)
            .fetch_all(self.pool)
            .await?;

        let mut results = Vec::new();
        for (id, name, slug, is_default) in dashboards {
            let widgets_rows = sqlx::query_as::<_, (String, String, String, Option<String>, String, i32, i32)>(
                "SELECT id, dashboard_id, type, title, config, width_span, sort_order FROM dashboard_widgets WHERE dashboard_id = ? ORDER BY sort_order ASC"
            )
            .bind(&id)
            .fetch_all(self.pool)
            .await?;

            let widgets = widgets_rows
                .into_iter()
                .map(|w| DashboardWidget {
                    id: w.0,
                    dashboard_id: w.1,
                    r#type: w.2,
                    title: w.3,
                    config: serde_json::from_str(&w.4).unwrap_or(json!({})),
                    width_span: w.5,
                    sort_order: w.6,
                })
                .collect();

            results.push(Dashboard {
                id,
                name,
                slug,
                is_default,
                widgets,
            });
        }
        Ok(results)
    }

    pub async fn add_widget(
        &self,
        widget_id: &str,
        dashboard_id: &str,
        w_type: &str,
        title: Option<&str>,
        config: &str,
        span: i32,
    ) -> Result<()> {
        // Verify dashboard ownership
        let exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM dashboards WHERE id = ? AND user_id = ?)",
        )
        .bind(dashboard_id)
        .bind(self.user_id)
        .fetch_one(self.pool)
        .await?;

        if !exists {
            return Err(crate::error::Error::NotFound(format!(
                "Dashboard {} not found",
                dashboard_id
            )));
        }

        sqlx::query("INSERT INTO dashboard_widgets (id, dashboard_id, type, title, config, width_span) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(widget_id)
            .bind(dashboard_id)
            .bind(w_type)
            .bind(title)
            .bind(config)
            .bind(span)
            .execute(self.pool)
            .await?;
        Ok(())
    }

    pub async fn delete_widget(&self, widget_id: &str) -> Result<()> {
        // Verify widget ownership through dashboard
        let exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM dashboard_widgets w JOIN dashboards d ON w.dashboard_id = d.id WHERE w.id = ? AND d.user_id = ?)"
        )
        .bind(widget_id)
        .bind(self.user_id)
        .fetch_one(self.pool)
        .await?;

        if !exists {
            return Err(crate::error::Error::NotFound(format!(
                "Widget {} not found",
                widget_id
            )));
        }

        sqlx::query("DELETE FROM dashboard_widgets WHERE id = ?")
            .bind(widget_id)
            .execute(self.pool)
            .await?;
        Ok(())
    }
}
