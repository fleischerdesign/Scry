use scry_plugin_sdk::prelude::*;

#[derive(Default)]
struct WeatherPlugin;

impl ScryPlugin for WeatherPlugin {
    fn get_manifest(&self) -> scry_plugin_sdk::Manifest {
        scry_plugin_sdk::Manifest {
            id: "scry-weather-plugin".to_string(),
            name: "Wetter Station".to_string(),
            version: "0.3.0".to_string(),
            description: "Lädt aktuelle Wetterdaten.".to_string(),
            subscriptions: vec![],
            capabilities: vec!["network".to_string(), "config".to_string()],
            exports: vec![
                scry_plugin_sdk::DataField { category: "weather.current".to_string(), path: "payload.temperature".to_string(), semantic_type: "environment.temperature".to_string(), description: "Aktuelle Temperatur".to_string() },
                scry_plugin_sdk::DataField { category: "weather.current".to_string(), path: "payload.condition".to_string(), semantic_type: "environment.condition".to_string(), description: "Wetterlage".to_string() },
            ],
            poll_interval: Some(300),
        }
    }

    fn on_ingest(&self, ev: scry_plugin_sdk::Event) -> Result<scry_plugin_sdk::Event, String> { Ok(ev) }

    fn get_reports(&self) -> Vec<scry_plugin_sdk::ReportMetadata> {
        vec![
            scry_plugin_sdk::ReportMetadata {
                id: "temp-history".to_string(),
                name: "Temperaturverlauf".to_string(),
                description: "Außentemperatur der letzten 7 Tage".to_string(),
                viz: scry_plugin_sdk::Visualization::LineChart,
            }
        ]
    }

    fn run_report(&self, report_id: &str) -> Result<scry_plugin_sdk::ReportData, String> {
        if report_id == "temp-history" {
            let stats = host::count_over_time("weather.current", "1d", 7);
            Ok(scry_plugin_sdk::ReportData {
                columns: vec!["Datum".to_string(), "Messungen".to_string()],
                data_json: serde_json::to_string(&stats).unwrap(),
            })
        } else {
            Err("Report nicht gefunden".to_string())
        }
    }

    fn on_poll(&self) -> Vec<scry_plugin_sdk::Event> {
        // Nutze globales Profil für die Stadt, falls vorhanden (Log-Demo)
        if let Some(city) = host::get_profile("location.city") {
            host::log_info(&format!("Polling weather for global city: {}", city));
        }

        let lat = host::get_config("latitude").unwrap_or_else(|| "52.52".to_string());
        let lon = host::get_config("longitude").unwrap_or_else(|| "13.41".to_string());
        let url = format!("https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current_weather=true", lat, lon);
        
        match host::http_get(&url) {
            Ok(text) => {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text)
                    && let Some(current) = json.get("current_weather") {
                        return vec![
                            scry_plugin_sdk::Event {
                                id: uuid::Uuid::new_v4(),
                                timestamp: chrono::Utc::now(),
                                category: "weather.current".to_string(),
                                source: "open-meteo-plugin".to_string(),
                                payload: current.clone(),
                                metadata: None,
                            }
                        ];
                    }
                vec![]
            },
            Err(_) => vec![]
        }
    }

    fn get_summary(&self, _start: &str, _end: &str) -> String {
        // Hier könnten wir eigentlich einen Join oder AVG über den Zeitraum machen,
        // für den Prototyp nehmen wir das aktuellste Event.
        let events = host::join_nearest("weather.current", "weather.current", 1);
        if let Some(first) = events.first()
            && let Some(base) = first.get("base") {
                let temp = base.get("temperature").and_then(|v| v.as_f64()).unwrap_or(0.0);
                return format!("The weather was around {:.1} degrees.", temp);
            }
        "Weather data unavailable.".to_string()
    }
}

scry_plugin!(WeatherPlugin);
