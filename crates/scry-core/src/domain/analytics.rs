use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

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

#[derive(Deserialize, IntoParams)]
pub struct SearchParams { pub q: String, pub limit: Option<u32>, pub offset: Option<u32> }

#[derive(Deserialize, IntoParams)]
pub struct CorrelateParams { 
    pub base_category: Option<String>, 
    pub join_category: Option<String>, 
    pub base_semantic: Option<String>, 
    pub join_semantic: Option<String>, 
    pub limit: Option<u32> 
}

#[derive(Deserialize, IntoParams, ToSchema)]
pub struct SemanticParams { 
    pub semantic_type: String, 
    pub limit: Option<u32>, 
    pub days: Option<u32>,
    pub interval: Option<String>,
}
