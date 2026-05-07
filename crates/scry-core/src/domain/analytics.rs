use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::{IntoParams, ToSchema};

#[derive(Serialize, ToSchema, TS)]
#[ts(export)]
pub struct CorrelationResult {
    pub base: serde_json::Value,
    pub joined: serde_json::Value,
}

#[derive(Serialize, ToSchema, TS)]
#[ts(export)]
pub struct SemanticStats {
    pub base_type: String,
    pub join_type: String,
    pub sample_size: usize,
    pub correlations: serde_json::Value,
}

#[derive(Deserialize, IntoParams, TS)]
#[ts(export)]
#[allow(dead_code)]
pub struct SearchParams {
    pub q: String,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Deserialize, IntoParams, TS)]
#[ts(export)]
pub struct CorrelateParams {
    pub base_category: Option<String>,
    pub join_category: Option<String>,
    pub base_semantic: Option<String>,
    pub join_semantic: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Deserialize, IntoParams, ToSchema, TS)]
#[ts(export)]
pub struct SemanticParams {
    pub semantic_type: String,
    pub limit: Option<u32>,
    pub days: Option<u32>,
    pub interval: Option<String>,
}
