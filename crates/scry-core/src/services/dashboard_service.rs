use sqlx::SqlitePool;
use uuid::Uuid;
use crate::models::Dashboard;
use crate::error::Result;
use crate::repository::DashboardRepository;

#[derive(Clone)]
pub struct DashboardService {
    db: SqlitePool,
}

impl DashboardService {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }

    pub async fn create_dashboard(&self, user_id: i64, name: &str) -> Result<()> {
        let repo = DashboardRepository::new(&self.db, user_id);
        let id = Uuid::new_v4().to_string();
        
        let slug = name.to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect::<String>()
            .replace("--", "-");
        
        repo.create(&id, name, &slug).await
    }

    pub async fn get_dashboards(&self, user_id: i64) -> Result<Vec<Dashboard>> {
        let repo = DashboardRepository::new(&self.db, user_id);
        repo.list().await
    }

    pub async fn add_widget(&self, user_id: i64, dashboard_id: &str, w_type: &str, title: Option<&str>, config: serde_json::Value) -> Result<()> {
        let repo = DashboardRepository::new(&self.db, user_id);
        let widget_id = Uuid::new_v4().to_string();
        let config_str = serde_json::to_string(&config).unwrap_or_else(|_| "{}".to_string());
        let span = config["width_span"].as_i64().unwrap_or(1) as i32;
        
        repo.add_widget(&widget_id, dashboard_id, w_type, title, &config_str, span).await
    }

    pub async fn delete_widget(&self, user_id: i64, widget_id: &str) -> Result<()> {
        let repo = DashboardRepository::new(&self.db, user_id);
        repo.delete_widget(widget_id).await
    }
}
