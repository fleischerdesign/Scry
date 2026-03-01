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
            capabilities: vec!["network".to_string(), "state".to_string(), "config".to_string()],
            exports: vec![
                scry_plugin_sdk::DataField { category: "music.scrobble".to_string(), path: "payload.artist".to_string(), semantic_type: "music.artist".to_string(), description: "Name des Künstlers".to_string() },
                scry_plugin_sdk::DataField { category: "music.scrobble".to_string(), path: "payload.track".to_string(), semantic_type: "music.track".to_string(), description: "Name des Songs".to_string() },
                scry_plugin_sdk::DataField { category: "music.scrobble".to_string(), path: "payload.album".to_string(), semantic_type: "music.album".to_string(), description: "Name des Albums".to_string() },
            ],
            provided_traits: vec![],
            poll_interval: Some(10),
            config_schema: None,
        }
    }

    async fn on_ingest(&self, mut ev: scry_plugin_sdk::Event) -> Result<scry_plugin_sdk::Event, String> {
        if ev.category == "music.scrobble" {
            let artist = ev.payload.get("artist").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string();
            
            // Tagge den Artist als semantische Entität
            ev.entities.push(scry_plugin_sdk::EntityRef {
                path: "payload.artist".to_string(),
                namespace: "scry.music".to_string(),
                typ: "artist".to_string(),
                id: artist,
            });

            let mut metadata = ev.metadata.unwrap_or(json!({}));
            if let Some(obj) = metadata.as_object_mut() {
                obj.insert("enriched_by".to_string(), json!("scry-semantic-v2"));
            }
            ev.metadata = Some(metadata);
            ev.source = format!("{}+bus", ev.source);
        }
        Ok(ev)
    }

    async fn get_reports(&self) -> Vec<scry_plugin_sdk::ReportMetadata> {
        vec![
            scry_plugin_sdk::ReportMetadata {
                id: "top-artists".to_string(),
                name: "Top Künstler".to_string(),
                description: "Deine meistgehörten Künstler".to_string(),
                viz: scry_plugin_sdk::Visualization::BarChart,
            }
        ]
    }

    async fn run_report(&self, report_id: &str) -> Result<scry_plugin_sdk::ReportData, String> {
        if report_id == "top-artists" {
            let stats = host::count_grouped("music.scrobble", "artist", 10).await;
            Ok(scry_plugin_sdk::ReportData {
                columns: vec!["Künstler".to_string(), "Scrobbles".to_string()],
                data_json: serde_json::to_string(&stats).unwrap(),
            })
        } else {
            Err("Report nicht gefunden".to_string())
        }
    }

    async fn on_poll(&self) -> Vec<scry_plugin_sdk::Event> {
        let current_count = host::get_state("poll_count").await.and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
        let new_count = current_count + 1;
        host::set_state("poll_count", &new_count.to_string()).await;

        let zen = match host::http_get("https://api.github.com/zen").await {
            Ok(text) => text,
            Err(_) => "Keep it clean.".to_string(),
        };

        vec![
            scry_plugin_sdk::Event {
                id: uuid::Uuid::new_v4(),
                timestamp: chrono::Utc::now(),
                category: "music.scrobble".to_string(),
                source: "bus-poller".to_string(),
                payload: json!({
                    "artist": "The Granular Poller",
                    "track": format!("Zen: {}", zen),
                    "album": "Scheduler Edition"
                }),
                metadata: None,
                entities: vec![],
            }
        ]
    }

    async fn get_summary(&self, _start: &str, _end: &str) -> String {
        let stats = host::count_grouped("music.scrobble", "artist", 3).await;
        if stats.is_empty() {
            return "No music scrobbled today.".to_string();
        }
        let artists: Vec<String> = stats.iter()
            .map(|v| v.get("key").and_then(|k| k.as_str()).unwrap_or("Unknown").to_string())
            .collect();
        format!("You listened to music by {}.", artists.join(", "))
    }
}

scry_plugin!(MusicPlugin);
