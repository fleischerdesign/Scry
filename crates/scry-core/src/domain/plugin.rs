use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct ApiReportMetadata {
    pub id: String,
    pub name: String,
    pub description: String,
    pub viz: String,
}

#[derive(Serialize, ToSchema)]
pub struct ApiReportData {
    pub columns: Vec<String>,
    pub data_json: String,
}

#[derive(Serialize, ToSchema)]
pub enum ApiWidgetTemplate { Metric, Trend, TopList, Status, Spotlight }

#[derive(Serialize, ToSchema)]
pub struct ApiWidgetDefinition {
    pub id: String,
    pub title: String,
    pub template: ApiWidgetTemplate,
    pub config_json: String,
}

#[derive(Serialize, ToSchema)]
pub struct ApiDataField {
    pub category: String,
    pub path: String,
    pub semantic_type: String,
    pub description: String,
    pub icon: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct ApiTraitCapability {
    pub entity_namespace: String,
    pub entity_type: String,
    pub trait_id: String,
}

#[derive(Serialize, ToSchema)]
pub struct PluginStatus {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub roles: Vec<String>,
    pub capabilities: Vec<String>,
    pub subscriptions: Vec<String>,
    pub exports: Vec<ApiDataField>,
    pub provided_traits: Vec<ApiTraitCapability>,
    pub reports: Vec<ApiReportMetadata>,
    pub config_schema: Option<String>,
    pub suggested_widgets: Vec<ApiWidgetDefinition>,
}

#[derive(Serialize, ToSchema)]
pub struct PluginReports {
    pub plugin: String,
    pub reports: Vec<ApiReportMetadata>,
}
