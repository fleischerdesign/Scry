use scry_plugin_sdk::prelude::*;
use scry_plugin_sdk::schema::{namespaces, traits};

#[derive(Default)]
struct MusicPlugin;

impl ScryPlugin for MusicPlugin {
    fn get_manifest(&self) -> scry_plugin_sdk::Manifest {
        scry_plugin_sdk::Manifest {
            id: "scry-music-plugin".to_string(),
            name: "Musik Simulator".to_string(),
            version: "0.4.0".to_string(),
            description: "Simuliert Musik-Events für Testzwecke mit dem neuen Identity-System.".to_string(),
            subscriptions: vec!["music.*".to_string()],
            capabilities: vec![
                "network".to_string(),
                "state".to_string(),
                "config".to_string(),
            ],
            exports: vec![
                scry_plugin_sdk::DataField {
                    category: "music.scrobble".to_string(),
                    path: "payload.energy_level".to_string(),
                    semantic_type: "metric.music.energy_level".to_string(),
                    description: "Simulierter Energie-Level des Songs".to_string(),
                    format: None,
                    icon: Some("lucide:zap".to_string()),
                    unit: Some("percent".to_string()),
                    privacy: None,
                    confidence: Some(0.8),
                    temporal: Some("absolute".to_string()),
                },
            ],
            domain_info: vec![scry_plugin_sdk::DomainInfo {
                ns: namespaces::MUSIC.to_string(),
                icon: Some("lucide:music".to_string()),
            }],
            predicates: vec![
                scry_plugin_sdk::PredicateDefinition {
                    id: "scry.music/played_by".to_string(),
                    label: "Played by".to_string(),
                    inverse_label: "Plays".to_string(),
                },
            ],
            provided_traits: vec![],
            poll_interval: Some(60),
            config_schema: None,
            suggested_widgets: vec![],
            oauth_config: None,
        }
    }

    fn on_ingest(&self, mut ev: SdkEvent) -> Result<SdkEvent, String> {
        if ev.category == "music.scrobble" {
            let artist = ev.payload.get("artist").and_then(|v| v.as_str()).unwrap_or("Simulator").to_string();
            let track = ev.payload.get("track").and_then(|v| v.as_str()).unwrap_or("Simulated Song").to_string();
            
            // 1. Deterministic IDs
            let track_id = identity::create_id(namespaces::MUSIC, &["track", &artist, &track]);
            let artist_id = identity::create_id(namespaces::MUSIC, &["artist", &artist]);

            ev.entities.push(scry_plugin_sdk::EntityRef {
                path: "payload.track".to_string(),
                namespace: namespaces::MUSIC.to_string(),
                typ: "track".to_string(),
                id: track_id.clone(),
            });
            ev.entities.push(scry_plugin_sdk::EntityRef {
                path: "payload.artist".to_string(),
                namespace: namespaces::MUSIC.to_string(),
                typ: "artist".to_string(),
                id: artist_id.clone(),
            });

            host::set_entity_trait(namespaces::MUSIC, "track", &track_id, traits::NAME, &json!(track).to_string());
            host::set_entity_trait(namespaces::MUSIC, "artist", &artist_id, traits::NAME, &json!(artist).to_string());

            ev.display_title = Some(track.clone());
            ev.display_subtitle = Some(format!("by {}", artist));

            // 2. Relationships
            host::set_relationship(scry_plugin_sdk::Relationship {
                source_namespace: namespaces::MUSIC.to_string(),
                source_type: "track".to_string(),
                source_id: track_id,
                predicate: "scry.music/played_by".to_string(),
                target_namespace: namespaces::MUSIC.to_string(),
                target_type: "artist".to_string(),
                target_id: artist_id,
            });
        }
        Ok(ev)
    }

    fn on_poll(&self) -> Vec<SdkEvent> {
        let energy_level = 42.0; // Fixed for now

        vec![SdkEvent {
            id: uuid::Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
            category: "music.scrobble".to_string(),
            source: "music-sim".to_string(),
            payload: json!({
                "artist": "The Simulator",
                "track": "A Beautiful Simulation",
                "energy_level": energy_level
            }),
            metadata: None,
            entities: vec![],
            context: vec!["alias:self".to_string()],
            context_info: None,
            display_image: None,
            display_value: None,
            display_title: Some("A Beautiful Simulation".to_string()),
            display_subtitle: Some("by The Simulator".to_string()),
            confidence: Some(1.0),
        }]
    }
}

scry_plugin!(MusicPlugin);
