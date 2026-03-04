use crate::plugins::scry::plugin::types::{
    DataField as WitDataField, DomainInfo as WitDomainInfo, EntityRef as WitEntityRef,
    Event as WitEvent, Manifest as WitManifest, PredicateDefinition as WitPredicateDefinition,
    ReportData as WitReportData, ReportMetadata as WitReportMetadata,
    TraitCapability as WitTraitCapability, Visualization as WitVisualization,
    WidgetDefinition as WitWidgetDefinition, WidgetTemplate as WitWidgetTemplate,
};
use chrono::{DateTime, Utc};
use scry_plugin_sdk::{
    DataField, DomainInfo, Manifest, PredicateDefinition, ReportData, ReportMetadata,
    TraitCapability, Visualization, WidgetDefinition, WidgetTemplate,
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

pub struct EventMapper;

impl EventMapper {
    pub fn from_wit(wit_ev: WitEvent) -> Result<Event, ConversionError> {
        Ok(Event {
            id: Uuid::parse_str(&wit_ev.id)
                .map_err(|e| ConversionError::InvalidUuid(e.to_string()))?,
            timestamp: DateTime::parse_from_rfc3339(&wit_ev.timestamp)
                .map_err(|e| ConversionError::InvalidTimestamp(e.to_string()))?
                .with_timezone(&Utc),
            category: wit_ev.category,
            source: wit_ev.source,
            payload: serde_json::from_str(&wit_ev.payload)
                .map_err(|e| ConversionError::InvalidPayload(e.to_string()))?,
            metadata: wit_ev.metadata.and_then(|m| serde_json::from_str(&m).ok()),
            entities: wit_ev
                .entities
                .into_iter()
                .map(|e| EntityMapper::from_wit(e))
                .collect(),
            context: wit_ev.context,
            context_info: wit_ev
                .context_info
                .and_then(|c| serde_json::from_str(&c).ok()),
            display_title: wit_ev.display_title,
            display_subtitle: wit_ev.display_subtitle,
            display_image: wit_ev.display_image,
            display_value: wit_ev.display_value,
            confidence: wit_ev.confidence,
        })
    }

    pub fn to_wit(event: &Event) -> WitEvent {
        WitEvent {
            id: event.id.to_string(),
            timestamp: event.timestamp.to_rfc3339(),
            category: event.category.clone(),
            source: event.source.clone(),
            payload: serde_json::to_string(&event.payload).unwrap(),
            metadata: event
                .metadata
                .as_ref()
                .map(|m| serde_json::to_string(m).unwrap()),
            entities: event.entities.iter().map(EntityMapper::to_wit).collect(),
            context: event.context.clone(),
            context_info: event
                .context_info
                .as_ref()
                .map(|c| serde_json::to_string(c).unwrap()),
            display_title: event.display_title.clone(),
            display_subtitle: event.display_subtitle.clone(),
            display_image: event.display_image.clone(),
            display_value: event.display_value,
            confidence: event.confidence,
        }
    }
}

pub struct EntityMapper;

impl EntityMapper {
    pub fn from_wit(wit_ent: WitEntityRef) -> EntityRef {
        EntityRef {
            path: wit_ent.path,
            namespace: wit_ent.namespace,
            typ: wit_ent.typ,
            id: wit_ent.id,
        }
    }

    pub fn to_wit(ent: &EntityRef) -> WitEntityRef {
        WitEntityRef {
            path: ent.path.clone(),
            namespace: ent.namespace.clone(),
            typ: ent.typ.clone(),
            id: ent.id.clone(),
        }
    }
}

pub struct ManifestMapper;

impl ManifestMapper {
    pub fn from_wit(wit_m: WitManifest) -> Manifest {
        Manifest {
            id: wit_m.id,
            name: wit_m.name,
            version: wit_m.version,
            description: wit_m.description,
            subscriptions: wit_m.subscriptions,
            capabilities: wit_m.capabilities,
            exports: wit_m
                .exports
                .into_iter()
                .map(ManifestMapper::data_field_from_wit)
                .collect(),
            domain_info: wit_m
                .domain_info
                .into_iter()
                .map(ManifestMapper::domain_info_from_wit)
                .collect(),
            predicates: wit_m
                .predicates
                .into_iter()
                .map(ManifestMapper::predicate_from_wit)
                .collect(),
            provided_traits: wit_m
                .provided_traits
                .into_iter()
                .map(ManifestMapper::trait_capability_from_wit)
                .collect(),
            poll_interval: wit_m.poll_interval,
            config_schema: wit_m.config_schema,
            suggested_widgets: wit_m
                .suggested_widgets
                .into_iter()
                .map(ManifestMapper::widget_def_from_wit)
                .collect(),
        }
    }

    fn data_field_from_wit(w: WitDataField) -> DataField {
        DataField {
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

    fn domain_info_from_wit(w: WitDomainInfo) -> DomainInfo {
        DomainInfo {
            ns: w.ns,
            icon: w.icon,
        }
    }

    fn predicate_from_wit(w: WitPredicateDefinition) -> PredicateDefinition {
        PredicateDefinition {
            id: w.id,
            label: w.label,
            inverse_label: w.inverse_label,
        }
    }

    fn trait_capability_from_wit(w: WitTraitCapability) -> TraitCapability {
        TraitCapability {
            entity_namespace: w.entity_namespace,
            entity_type: w.entity_type,
            trait_id: w.trait_id,
        }
    }

    fn widget_def_from_wit(w: WitWidgetDefinition) -> WidgetDefinition {
        WidgetDefinition {
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

    pub fn to_wit(m: &Manifest) -> WitManifest {
        WitManifest {
            id: m.id.clone(),
            name: m.name.clone(),
            version: m.version.clone(),
            description: m.description.clone(),
            subscriptions: m.subscriptions.clone(),
            capabilities: m.capabilities.clone(),
            exports: m
                .exports
                .iter()
                .map(ManifestMapper::data_field_to_wit)
                .collect(),
            domain_info: m
                .domain_info
                .iter()
                .map(ManifestMapper::domain_info_to_wit)
                .collect(),
            predicates: m
                .predicates
                .iter()
                .map(ManifestMapper::predicate_to_wit)
                .collect(),
            provided_traits: m
                .provided_traits
                .iter()
                .map(ManifestMapper::trait_capability_to_wit)
                .collect(),
            poll_interval: m.poll_interval,
            config_schema: m.config_schema.clone(),
            suggested_widgets: m
                .suggested_widgets
                .iter()
                .map(ManifestMapper::widget_def_to_wit)
                .collect(),
        }
    }

    fn data_field_to_wit(w: &DataField) -> WitDataField {
        WitDataField {
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

    fn domain_info_to_wit(w: &DomainInfo) -> WitDomainInfo {
        WitDomainInfo {
            ns: w.ns.clone(),
            icon: w.icon.clone(),
        }
    }

    fn predicate_to_wit(w: &PredicateDefinition) -> WitPredicateDefinition {
        WitPredicateDefinition {
            id: w.id.clone(),
            label: w.label.clone(),
            inverse_label: w.inverse_label.clone(),
        }
    }

    fn trait_capability_to_wit(w: &TraitCapability) -> WitTraitCapability {
        WitTraitCapability {
            entity_namespace: w.entity_namespace.clone(),
            entity_type: w.entity_type.clone(),
            trait_id: w.trait_id.clone(),
        }
    }

    fn widget_def_to_wit(w: &WidgetDefinition) -> WitWidgetDefinition {
        WitWidgetDefinition {
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

pub struct ReportMapper;

impl ReportMapper {
    pub fn from_wit(w: WitReportMetadata) -> ReportMetadata {
        ReportMetadata {
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

    pub fn to_wit(r: &ReportMetadata) -> WitReportMetadata {
        WitReportMetadata {
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

    pub fn report_data_from_wit(w: WitReportData) -> ReportData {
        ReportData {
            columns: w.columns,
            data_json: w.data_json,
        }
    }

    pub fn report_data_to_wit(r: &ReportData) -> WitReportData {
        WitReportData {
            columns: r.columns.clone(),
            data_json: r.data_json.clone(),
        }
    }
}
