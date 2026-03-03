use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub enum SemanticCategory {
    Metric,
    Entity,
    State,
    Unknown,
}

impl From<&str> for SemanticCategory {
    fn from(s: &str) -> Self {
        if s.starts_with("metric.") {
            SemanticCategory::Metric
        } else if s.starts_with("entity.") {
            SemanticCategory::Entity
        } else if s.starts_with("state.") {
            SemanticCategory::State
        } else {
            SemanticCategory::Unknown
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SemanticMapping {
    pub scry_type: String,
    pub category: SemanticCategory,
    pub schema_org_uri: Option<String>,
    pub description: Option<String>,
}

pub struct SemanticResolver;

impl SemanticResolver {
    /// Maps a Scry semantic type or trait to a schema.org URI.
    pub fn resolve_to_schema_org(scry_type: &str) -> Option<String> {
        // ... (existing match)
        let uri = match scry_type {
            // Core Traits
            "scry.core/name" => "https://schema.org/name",
            "scry.core/description" => "https://schema.org/description",
            "scry.visual/photo" | "scry.core/avatar" => "https://schema.org/image",
            "scry.core/city" => "https://schema.org/addressLocality",
            
            // New Semantic Entity Types
            "entity.music.artist" => "https://schema.org/MusicGroup",
            "entity.music.album" => "https://schema.org/MusicAlbum",
            "entity.music.track" => "https://schema.org/MusicRecording",
            "entity.core.user" => "https://schema.org/Person",
            
            // New Semantic Metric Types
            "metric.environment.temperature" => "https://schema.org/QuantitativeValue", // Specific properties like value/unitCode apply here
            
            _ => return None,
        };
        Some(uri.to_string())
    }

    /// Provides full metadata for a given Scry semantic type.
    pub fn get_mapping(scry_type: &str) -> SemanticMapping {
        SemanticMapping {
            scry_type: scry_type.to_string(),
            category: SemanticCategory::from(scry_type),
            schema_org_uri: Self::resolve_to_schema_org(scry_type),
            description: Some(format!("Semantic mapping for {}", scry_type)),
        }
    }
}
