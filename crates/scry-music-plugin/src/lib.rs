use scry_plugin_sdk::prelude::*;

#[derive(Default)]
struct MusicPlugin;

impl ScryPlugin for MusicPlugin {
    fn get_manifest(&self) -> scry_plugin_sdk::Manifest {
        scry_plugin_sdk::Manifest {
            id: "scry-music-plugin".to_string(),
            name: "Musik Scrobbler".to_string(),
            version: "0.3.0".to_string(),
            description: "Importiert und analysiert deine Musik.".to_string(),
            subscriptions: vec!["music.*".to_string()],
            capabilities: vec![
                "network".to_string(),
                "state".to_string(),
                "config".to_string(),
            ],
            exports: vec![
                scry_plugin_sdk::DataField {
                    category: "music.scrobble".to_string(),
                    path: "payload.artist".to_string(),
                    semantic_type: "entity.music.artist".to_string(),
                    description: "Name des Künstlers".to_string(),
                    format: None,
                    icon: Some("lucide:mic".to_string()),
                    unit: None,
                    privacy: Some("pii".to_string()),
                    confidence: Some(1.0),
                    temporal: None,
                },
                scry_plugin_sdk::DataField {
                    category: "music.scrobble".to_string(),
                    path: "payload.track".to_string(),
                    semantic_type: "entity.music.track".to_string(),
                    description: "Name des Songs".to_string(),
                    format: None,
                    icon: Some("lucide:music".to_string()),
                    unit: None,
                    privacy: None,
                    confidence: Some(1.0),
                    temporal: None,
                },
                scry_plugin_sdk::DataField {
                    category: "music.scrobble".to_string(),
                    path: "payload.album".to_string(),
                    semantic_type: "entity.music.album".to_string(),
                    description: "Name des Albums".to_string(),
                    format: None,
                    icon: Some("lucide:disc".to_string()),
                    unit: None,
                    privacy: None,
                    confidence: Some(1.0),
                    temporal: None,
                },
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
                ns: "scry.music".to_string(),
                icon: Some("lucide:music".to_string()),
            }],
            predicates: vec![
                scry_plugin_sdk::PredicateDefinition {
                    id: "scry.music/by_artist".to_string(),
                    label: "By Artist".to_string(),
                    inverse_label: "Discography / Works".to_string(),
                },
                scry_plugin_sdk::PredicateDefinition {
                    id: "scry.music/belongs_to_album".to_string(),
                    label: "Part of Album".to_string(),
                    inverse_label: "Contains Tracks".to_string(),
                },
            ],
            provided_traits: vec![],
            poll_interval: Some(10),
            config_schema: None,
            suggested_widgets: vec![scry_plugin_sdk::WidgetDefinition {
                id: "music-top-artists".to_string(),
                title: "Top Artists (30D)".to_string(),
                template: scry_plugin_sdk::WidgetTemplate::TopList,
                config_json: json!({ "semantic_type": "entity.music.artist", "days": 30 })
                    .to_string(),
            }],
            oauth_config: None,
        }
    }

    fn on_ingest(&self, mut ev: scry_plugin_sdk::Event) -> Result<scry_plugin_sdk::Event, String> {
        if ev.category == "music.scrobble" {
            let artist = ev
                .payload
                .get("artist")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string();
            let track = ev
                .payload
                .get("track")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string();
            let album = ev
                .payload
                .get("album")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string();

            // 1. Tagge Artist, Track und Album als semantische Entitäten
            ev.entities.push(scry_plugin_sdk::EntityRef {
                path: "payload.artist".to_string(),
                namespace: "scry.music".to_string(),
                typ: "artist".to_string(),
                id: artist.clone(),
            });
            ev.entities.push(scry_plugin_sdk::EntityRef {
                path: "payload.track".to_string(),
                namespace: "scry.music".to_string(),
                typ: "track".to_string(),
                id: track.clone(),
            });
            ev.entities.push(scry_plugin_sdk::EntityRef {
                path: "payload.album".to_string(),
                namespace: "scry.music".to_string(),
                typ: "album".to_string(),
                id: album.clone(),
            });

            ev.display_title = Some(format!("{} - {}", artist, track));
            ev.display_subtitle = Some(format!("Album: {}", album));

            // 2. Beziehungen im Knowledge Graph hinterlegen
            let track_rel = scry_plugin_sdk::Relationship {
                source_namespace: "scry.music".to_string(),
                source_type: "track".to_string(),
                source_id: track.clone(),
                predicate: "scry.music/belongs_to_album".to_string(),
                target_namespace: "scry.music".to_string(),
                target_type: "album".to_string(),
                target_id: album.clone(),
            };
            let artist_rel = scry_plugin_sdk::Relationship {
                source_namespace: "scry.music".to_string(),
                source_type: "track".to_string(),
                source_id: track.clone(),
                predicate: "scry.music/by_artist".to_string(),
                target_namespace: "scry.music".to_string(),
                target_type: "artist".to_string(),
                target_id: artist.clone(),
            };

            let _ = host::set_relationship(track_rel);
            let _ = host::set_relationship(artist_rel);

            let mut metadata = ev.metadata.unwrap_or(json!({}));
            if let Some(obj) = metadata.as_object_mut() {
                obj.insert("enriched_by".to_string(), json!("scry-semantic-v2"));
            }
            ev.metadata = Some(metadata);
            ev.source = format!("{}+bus", ev.source);
        }
        Ok(ev)
    }

    fn get_reports(&self) -> Vec<scry_plugin_sdk::ReportMetadata> {
        vec![scry_plugin_sdk::ReportMetadata {
            id: "top-artists".to_string(),
            name: "Top Künstler".to_string(),
            description: "Deine meistgehörten Künstler".to_string(),
            viz: scry_plugin_sdk::Visualization::BarChart,
        }]
    }

    fn run_report(&self, report_id: &str) -> Result<scry_plugin_sdk::ReportData, String> {
        if report_id == "top-artists" {
            let stats = host::count_grouped("music.scrobble", "artist", 10);
            Ok(scry_plugin_sdk::ReportData {
                columns: vec!["Künstler".to_string(), "Scrobbles".to_string()],
                data_json: serde_json::to_string(&stats).unwrap(),
            })
        } else {
            Err("Report nicht gefunden".to_string())
        }
    }

    fn on_poll(&self) -> Vec<scry_plugin_sdk::Event> {
        let current_count = host::get_state("poll_count")
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0);
        let new_count = current_count + 1;
        host::set_state("poll_count", &new_count.to_string());

        let zen = match host::http_get("https://api.github.com/zen") {
            Ok(text) => text,
            Err(_) => "Keep it clean.".to_string(),
        };

        let energy_level = (new_count % 10) as f64 * 10.0;

        vec![scry_plugin_sdk::Event {
            id: uuid::Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
            category: "music.scrobble".to_string(),
            source: "bus-poller".to_string(),
            payload: json!({
                "artist": "The Granular Poller",
                "track": format!("Zen: {}", zen),
                "album": "Scheduler Edition",
                "energy_level": energy_level
            }),
            metadata: None,
            entities: vec![],
            context: vec!["alias:self".to_string()],
            context_info: None,
            display_title: Some(format!("The Granular Poller - Zen: {}", zen)),
            display_subtitle: Some("Album: Scheduler Edition".to_string()),
            display_image: None,
            display_value: None,
            confidence: Some(1.0),
        }]
    }

    fn get_summary(&self, _start: &str, _end: &str) -> String {
        let stats = host::count_grouped("music.scrobble", "artist", 3);
        if stats.is_empty() {
            return "No music scrobbled today.".to_string();
        }
        let artists: Vec<String> = stats
            .iter()
            .map(|v| {
                v.get("key")
                    .and_then(|k| k.as_str())
                    .unwrap_or("Unknown")
                    .to_string()
            })
            .collect();
        format!("You listened to music by {}.", artists.join(", "))
    }
}

scry_plugin!(MusicPlugin);
