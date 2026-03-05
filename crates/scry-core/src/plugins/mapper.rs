use crate::plugins::scry::plugin::types::{
    DataField as WitDataField, DomainInfo as WitDomainInfo, EntityRef as WitEntityRef,
    Event as WitEvent, Manifest as WitManifest, OauthConfig as WitOauthConfig,
    PredicateDefinition as WitPredicateDefinition, ReportData as WitReportData,
    ReportMetadata as WitReportMetadata, TraitCapability as WitTraitCapability,
    Visualization as WitVisualization, WidgetDefinition as WitWidgetDefinition,
    WidgetTemplate as WitWidgetTemplate,
};
use chrono::{DateTime, Utc};
use scry_plugin_sdk::{
    DataField, DomainInfo, Manifest, OAuthConfig as SdkOAuthConfig, PredicateDefinition,
    ReportData, ReportMetadata, TraitCapability, Visualization, WidgetDefinition, WidgetTemplate,
};
use scry_proto::{EntityRef, Event};
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum ConversionError {
    #[error("Invalid UUID: {0}")]
    InvalidUuid(String),
    #[error("Invalid timestamp: {0}")]
    InvalidTimestamp(String),
    #[error("Invalid payload JSON: {0}")]
    InvalidPayload(String),
    #[error("Plugin error: {0}")]
    Plugin(String),
}

// --- Event Conversions ---

impl TryFrom<WitEvent> for Event {
    type Error = ConversionError;

    fn try_from(wit_ev: WitEvent) -> Result<Self, Self::Error> {
        Ok(Event {
            id: Uuid::parse_str(&wit_ev.id)
                .map_err(|e| ConversionError::InvalidUuid(e.to_string()))?,
            timestamp: DateTime::parse_from_rfc3339(&wit_ev.timestamp)
                .map_err(|e| ConversionError::InvalidTimestamp(e.to_string()))?
                .with_timezone(&Utc),
            category: wit_ev.category,
            source: wit_ev.source,
            payload: serde_json::from_str(&wit_ev.payload)
                .map_err(|e| ConversionError::InvalidPayload(format!("Payload: {}", e)))?,
            metadata: match wit_ev.metadata {
                Some(m) if !m.is_empty() => Some(
                    serde_json::from_str(&m)
                        .map_err(|e| ConversionError::InvalidPayload(format!("Metadata: {}", e)))?,
                ),
                _ => None,
            },
            entities: wit_ev.entities.into_iter().map(EntityRef::from).collect(),
            context: wit_ev.context,
            context_info: match wit_ev.context_info {
                Some(c) if !c.is_empty() => Some(serde_json::from_str(&c).map_err(|e| {
                    ConversionError::InvalidPayload(format!("Context Info: {}", e))
                })?),
                _ => None,
            },
            display_title: wit_ev.display_title,
            display_subtitle: wit_ev.display_subtitle,
            display_image: wit_ev.display_image,
            display_value: wit_ev.display_value,
            confidence: wit_ev.confidence,
        })
    }
}

impl From<&Event> for WitEvent {
    fn from(event: &Event) -> Self {
        WitEvent {
            id: event.id.to_string(),
            timestamp: event.timestamp.to_rfc3339(),
            category: event.category.clone(),
            source: event.source.clone(),
            payload: serde_json::to_string(&event.payload).unwrap_or_default(),
            metadata: event
                .metadata
                .as_ref()
                .map(|m| serde_json::to_string(m).unwrap_or_default()),
            entities: event.entities.iter().map(WitEntityRef::from).collect(),
            context: event.context.clone(),
            context_info: event
                .context_info
                .as_ref()
                .map(|c| serde_json::to_string(c).unwrap_or_default()),
            display_title: event.display_title.clone(),
            display_subtitle: event.display_subtitle.clone(),
            display_image: event.display_image.clone(),
            display_value: event.display_value.clone(),
            confidence: event.confidence,
        }
    }
}

// --- Entity Conversions ---

impl From<WitEntityRef> for EntityRef {
    fn from(wit_ent: WitEntityRef) -> Self {
        EntityRef {
            path: wit_ent.path,
            namespace: wit_ent.namespace,
            typ: wit_ent.typ,
            id: wit_ent.id,
        }
    }
}

impl From<&EntityRef> for WitEntityRef {
    fn from(ent: &EntityRef) -> Self {
        WitEntityRef {
            path: ent.path.clone(),
            namespace: ent.namespace.clone(),
            typ: ent.typ.clone(),
            id: ent.id.clone(),
        }
    }
}

// --- Manifest Conversions ---

impl From<WitManifest> for Manifest {
    fn from(wit_m: WitManifest) -> Self {
        Manifest {
            id: wit_m.id,
            name: wit_m.name,
            version: wit_m.version,
            description: wit_m.description,
            subscriptions: wit_m.subscriptions,
            capabilities: wit_m.capabilities,
            exports: wit_m.exports.into_iter().map(DataField::from).collect(),
            domain_info: wit_m
                .domain_info
                .into_iter()
                .map(DomainInfo::from)
                .collect(),
            predicates: wit_m
                .predicates
                .into_iter()
                .map(PredicateDefinition::from)
                .collect(),
            provided_traits: wit_m
                .provided_traits
                .into_iter()
                .map(TraitCapability::from)
                .collect(),
            poll_interval: wit_m.poll_interval,
            config_schema: wit_m.config_schema,
            suggested_widgets: wit_m
                .suggested_widgets
                .into_iter()
                .map(WidgetDefinition::from)
                .collect(),
            oauth_config: wit_m.oauth_config.as_ref().map(|o| SdkOAuthConfig {
                auth_url: o.auth_url.clone(),
                token_url: o.token_url.clone(),
                scopes: o.scopes.clone(),
            }),
        }
    }
}

impl From<&Manifest> for WitManifest {
    fn from(m: &Manifest) -> Self {
        WitManifest {
            id: m.id.clone(),
            name: m.name.clone(),
            version: m.version.clone(),
            description: m.description.clone(),
            subscriptions: m.subscriptions.clone(),
            capabilities: m.capabilities.clone(),
            exports: m.exports.iter().map(WitDataField::from).collect(),
            domain_info: m.domain_info.iter().map(WitDomainInfo::from).collect(),
            predicates: m
                .predicates
                .iter()
                .map(WitPredicateDefinition::from)
                .collect(),
            provided_traits: m
                .provided_traits
                .iter()
                .map(WitTraitCapability::from)
                .collect(),
            poll_interval: m.poll_interval,
            config_schema: m.config_schema.clone(),
            suggested_widgets: m
                .suggested_widgets
                .iter()
                .map(WitWidgetDefinition::from)
                .collect(),
            oauth_config: m.oauth_config.as_ref().map(|o| WitOauthConfig {
                auth_url: o.auth_url.clone(),
                token_url: o.token_url.clone(),
                scopes: o.scopes.clone(),
            }),
        }
    }
}

// Helper conversions for Manifest nested types
impl From<WitDataField> for DataField {
    fn from(w: WitDataField) -> Self {
        Self {
            category: w.category,
            path: w.path,
            semantic_type: w.semantic_type,
            description: w.description,
            format: w.format,
            icon: w.icon,
            unit: w.unit,
            privacy: w.privacy,
            confidence: w.confidence,
            temporal: w.temporal,
        }
    }
}

impl From<&DataField> for WitDataField {
    fn from(w: &DataField) -> Self {
        Self {
            category: w.category.clone(),
            path: w.path.clone(),
            semantic_type: w.semantic_type.clone(),
            description: w.description.clone(),
            format: w.format.clone(),
            icon: w.icon.clone(),
            unit: w.unit.clone(),
            privacy: w.privacy.clone(),
            confidence: w.confidence,
            temporal: w.temporal.clone(),
        }
    }
}

impl From<WitDomainInfo> for DomainInfo {
    fn from(w: WitDomainInfo) -> Self {
        Self {
            ns: w.ns,
            icon: w.icon,
        }
    }
}

impl From<&DomainInfo> for WitDomainInfo {
    fn from(w: &DomainInfo) -> Self {
        Self {
            ns: w.ns.clone(),
            icon: w.icon.clone(),
        }
    }
}

impl From<WitPredicateDefinition> for PredicateDefinition {
    fn from(w: WitPredicateDefinition) -> Self {
        Self {
            id: w.id,
            label: w.label,
            inverse_label: w.inverse_label,
        }
    }
}

impl From<&PredicateDefinition> for WitPredicateDefinition {
    fn from(w: &PredicateDefinition) -> Self {
        Self {
            id: w.id.clone(),
            label: w.label.clone(),
            inverse_label: w.inverse_label.clone(),
        }
    }
}

impl From<WitTraitCapability> for TraitCapability {
    fn from(w: WitTraitCapability) -> Self {
        Self {
            entity_namespace: w.entity_namespace,
            entity_type: w.entity_type,
            trait_id: w.trait_id,
        }
    }
}

impl From<&TraitCapability> for WitTraitCapability {
    fn from(w: &TraitCapability) -> Self {
        Self {
            entity_namespace: w.entity_namespace.clone(),
            entity_type: w.entity_type.clone(),
            trait_id: w.trait_id.clone(),
        }
    }
}

impl From<WitWidgetDefinition> for WidgetDefinition {
    fn from(w: WitWidgetDefinition) -> Self {
        Self {
            id: w.id,
            title: w.title,
            template: match w.template {
                WitWidgetTemplate::Metric => WidgetTemplate::Metric,
                WitWidgetTemplate::Trend => WidgetTemplate::Trend,
                WitWidgetTemplate::TopList => WidgetTemplate::TopList,
                WitWidgetTemplate::Status => WidgetTemplate::Status,
                WitWidgetTemplate::Spotlight => WidgetTemplate::Spotlight,
            },
            config_json: w.config_json,
        }
    }
}

impl From<&WidgetDefinition> for WitWidgetDefinition {
    fn from(w: &WidgetDefinition) -> Self {
        Self {
            id: w.id.clone(),
            title: w.title.clone(),
            template: match w.template {
                WidgetTemplate::Metric => WitWidgetTemplate::Metric,
                WidgetTemplate::Trend => WitWidgetTemplate::Trend,
                WidgetTemplate::TopList => WitWidgetTemplate::TopList,
                WidgetTemplate::Status => WitWidgetTemplate::Status,
                WidgetTemplate::Spotlight => WitWidgetTemplate::Spotlight,
            },
            config_json: w.config_json.clone(),
        }
    }
}

// --- Report Conversions ---

impl From<WitReportMetadata> for ReportMetadata {
    fn from(w: WitReportMetadata) -> Self {
        Self {
            id: w.id,
            name: w.name,
            description: w.description,
            viz: match w.viz {
                WitVisualization::Table => Visualization::Table,
                WitVisualization::BarChart => Visualization::BarChart,
                WitVisualization::LineChart => Visualization::LineChart,
                WitVisualization::PieChart => Visualization::PieChart,
            },
        }
    }
}

impl From<&ReportMetadata> for WitReportMetadata {
    fn from(r: &ReportMetadata) -> Self {
        Self {
            id: r.id.clone(),
            name: r.name.clone(),
            description: r.description.clone(),
            viz: match r.viz {
                Visualization::Table => WitVisualization::Table,
                Visualization::BarChart => WitVisualization::BarChart,
                Visualization::LineChart => WitVisualization::LineChart,
                Visualization::PieChart => WitVisualization::PieChart,
            },
        }
    }
}

impl From<WitReportData> for ReportData {
    fn from(w: WitReportData) -> Self {
        Self {
            columns: w.columns,
            data_json: w.data_json,
        }
    }
}

impl From<&ReportData> for WitReportData {
    fn from(r: &ReportData) -> Self {
        Self {
            columns: r.columns.clone(),
            data_json: r.data_json.clone(),
        }
    }
}
