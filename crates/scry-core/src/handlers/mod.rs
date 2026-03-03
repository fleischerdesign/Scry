pub mod auth;
pub mod events;
pub mod dashboards;
pub mod plugins;
pub mod analytics;
pub mod entities;
pub mod system;

pub use auth::*;
pub use events::*;
pub use dashboards::*;
pub use plugins::*;
pub use analytics::*;
pub use entities::*;
pub use system::*;

use serde::Deserialize;

#[derive(Deserialize, utoipa::IntoParams)]
pub struct ListParams { pub category: Option<String>, pub limit: Option<u32>, pub offset: Option<u32> }

#[derive(Deserialize, utoipa::IntoParams)]
pub struct SearchParams { pub q: String, pub limit: Option<u32>, pub offset: Option<u32> }

#[derive(Deserialize, utoipa::IntoParams)]
pub struct SummaryParams { pub date: Option<String> }

#[derive(Deserialize, utoipa::IntoParams)]
pub struct CorrelateParams { pub base_category: Option<String>, pub join_category: Option<String>, pub base_semantic: Option<String>, pub join_semantic: Option<String>, pub limit: Option<u32> }

#[derive(Deserialize, utoipa::IntoParams, utoipa::ToSchema)]
pub struct SemanticParams { 
    pub semantic_type: String, 
    pub limit: Option<u32>, 
    pub days: Option<u32>,
    pub interval: Option<String>,
}
