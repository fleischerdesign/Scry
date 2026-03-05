use serde::Serialize;
use utoipa::ToSchema;
use ts_rs::TS;

#[derive(Serialize, ToSchema, TS)]
#[ts(export)]
pub struct ApiReportMetadata {
    pub id: String,
    pub name: String,
    pub description: String,
    pub viz: String,
}

impl From<scry_plugin_sdk::ReportMetadata> for ApiReportMetadata {
    fn from(m: scry_plugin_sdk::ReportMetadata) -> Self {
        Self {
            id: m.id,
            name: m.name,
            description: m.description,
            viz: format!("{:?}", m.viz),
        }
    }
}

#[derive(Serialize, ToSchema, TS)]
#[ts(export)]
pub struct ApiReportData {
    pub columns: Vec<String>,
    pub data_json: String,
}

impl From<scry_plugin_sdk::ReportData> for ApiReportData {
    fn from(d: scry_plugin_sdk::ReportData) -> Self {
        Self {
            columns: d.columns,
            data_json: d.data_json,
        }
    }
}

#[derive(Serialize, ToSchema, TS)]
#[ts(export)]
pub enum ApiWidgetTemplate { Metric, Trend, TopList, Status, Spotlight }

impl From<scry_plugin_sdk::WidgetTemplate> for ApiWidgetTemplate {
    fn from(t: scry_plugin_sdk::WidgetTemplate) -> Self {
        match t {
            scry_plugin_sdk::WidgetTemplate::Metric => Self::Metric,
            scry_plugin_sdk::WidgetTemplate::Trend => Self::Trend,
            scry_plugin_sdk::WidgetTemplate::TopList => Self::TopList,
            scry_plugin_sdk::WidgetTemplate::Status => Self::Status,
            scry_plugin_sdk::WidgetTemplate::Spotlight => Self::Spotlight,
        }
    }
}

#[derive(Serialize, ToSchema, TS)]
#[ts(export)]
pub struct ApiWidgetDefinition {
    pub id: String,
    pub title: String,
    pub template: ApiWidgetTemplate,
    pub config_json: String,
}

impl From<scry_plugin_sdk::WidgetDefinition> for ApiWidgetDefinition {
    fn from(w: scry_plugin_sdk::WidgetDefinition) -> Self {
        Self {
            id: w.id,
            title: w.title,
            template: w.template.into(),
            config_json: w.config_json,
        }
    }
}

#[derive(Serialize, ToSchema, TS)]
#[ts(export)]
pub struct ApiDataField {
    pub category: String,
    pub path: String,
    pub semantic_type: String,
    pub description: String,
    pub icon: Option<String>,
    pub unit: Option<String>,
    pub privacy: Option<String>,
    pub confidence: Option<f64>,
    pub temporal: Option<String>,
}

impl From<scry_plugin_sdk::DataField> for ApiDataField {
    fn from(f: scry_plugin_sdk::DataField) -> Self {
        Self {
            category: f.category,
            path: f.path,
            semantic_type: f.semantic_type,
            description: f.description,
            icon: f.icon,
            unit: f.unit,
            privacy: f.privacy,
            confidence: f.confidence,
            temporal: f.temporal,
        }
    }
}

#[derive(Serialize, ToSchema, TS)]
#[ts(export)]
pub struct ApiTraitCapability {
    pub entity_namespace: String,
    pub entity_type: String,
    pub trait_id: String,
}

impl From<scry_plugin_sdk::TraitCapability> for ApiTraitCapability {
    fn from(t: scry_plugin_sdk::TraitCapability) -> Self {
        Self {
            entity_namespace: t.entity_namespace,
            entity_type: t.entity_type,
            trait_id: t.trait_id,
        }
    }
}

#[derive(Serialize, ToSchema, TS)]
#[ts(export)]
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

impl PluginStatus {
    pub fn from_sdk(id: String, m: scry_plugin_sdk::Manifest, reports: Vec<scry_plugin_sdk::ReportMetadata>) -> Self {
        Self {
            id,
            name: m.name,
            version: m.version,
            description: m.description,
            roles: Vec::new(), // Set by caller based on context
            capabilities: m.capabilities,
            subscriptions: m.subscriptions,
            exports: m.exports.into_iter().map(|e| e.into()).collect(),
            provided_traits: m.provided_traits.into_iter().map(|t| t.into()).collect(),
            reports: reports.into_iter().map(|r| r.into()).collect(),
            config_schema: m.config_schema,
            suggested_widgets: m.suggested_widgets.into_iter().map(|w| w.into()).collect(),
        }
    }
}

#[derive(Serialize, ToSchema, TS)]
#[ts(export)]
pub struct PluginReports {
    pub plugin: String,
    pub reports: Vec<ApiReportMetadata>,
}
