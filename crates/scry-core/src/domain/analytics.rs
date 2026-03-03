use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct CorrelationResult {
    pub base: serde_json::Value,
    pub joined: serde_json::Value,
}

#[derive(Serialize, ToSchema)]
pub struct SemanticStats {
    pub base_type: String,
    pub join_type: String,
    pub sample_size: usize,
    pub correlations: serde_json::Value,
}
