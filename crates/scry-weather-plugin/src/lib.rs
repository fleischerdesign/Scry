use scry_plugin_sdk::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Default)]
struct WeatherPlugin;

#[derive(Debug, Deserialize)]
struct GeoLocation {
    latitude: f64,
    longitude: f64,
}

impl ScryPlugin for WeatherPlugin {
    fn get_manifest(&self) -> scry_plugin_sdk::Manifest {
        scry_plugin_sdk::Manifest {
            id: "scry-weather-plugin".to_string(),
            name: "Weather Node".to_string(),
            version: "0.2.0".to_string(),
            description: "Abfrage von Wetterdaten basierend auf dem aktuellen Standort des Users.".to_string(),
            subscriptions: vec![],
            capabilities: vec!["network".to_string(), "state".to_string(), "config".to_string()],
            exports: vec![
                scry_plugin_sdk::DataField {
                    category: "weather.current".to_string(),
                    path: "temperature".to_string(),
                    semantic_type: "environment.temperature".to_string(),
                    description: "Aktuelle Temperatur in Celsius".to_string(),
                }
            ],
            provided_traits: vec![],
            poll_interval: Some(600), // Alle 10 Minuten
            config_schema: Some(json!({
                "type": "object",
                "properties": {
                    "latitude": { "type": "number", "description": "Manuelle Latitude (optional)" },
                    "longitude": { "type": "number", "description": "Manuelle Longitude (optional)" }
                }
            }).to_string()),
        }
    }

    async fn on_poll(&self) -> Vec<SdkEvent> {
        host::log_info("Weather: Polling...").await;

        // 1. Versuche Koordinaten aus der lokalen Config zu laden
        let mut lat = host::get_config("latitude").await.and_then(|v| v.parse::<f64>().ok());
        let mut lon = host::get_config("longitude").await.and_then(|v| v.parse::<f64>().ok());

        // 2. Fallback: Versuche den aktuellen Standort-Trait vom User 'self' zu lesen
        if lat.is_none() || lon.is_none() {
            if let Some(loc_json) = host::get_entity_trait("scry.core", "user", "self", "scry.geo/location").await {
                if let Ok(loc) = serde_json::from_str::<GeoLocation>(&loc_json) {
                    host::log_info(&format!("Weather: Using dynamic user location: {}, {}", loc.latitude, loc.longitude)).await;
                    lat = Some(loc.latitude);
                    lon = Some(loc.longitude);
                }
            }
        }

        // 3. Letzter Fallback: Fixe Koordinaten (z.B. Berlin), falls gar nichts gefunden wurde
        let lat = lat.unwrap_or(52.52);
        let lon = lon.unwrap_or(13.41);

        // API Abfrage (Open-Meteo)
        let url = format!("https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current_weather=true", lat, lon);
        
        match host::http_get(&url).await {
            Ok(resp_json) => {
                let v: serde_json::Value = serde_json::from_str(&resp_json).unwrap_or_default();
                let temp = v["current_weather"]["temperature"].as_f64().unwrap_or(0.0);
                
                vec![SdkEvent {
                    id: uuid::Uuid::new_v4(),
                    timestamp: chrono::Utc::now(),
                    category: "weather.current".to_string(),
                    source: "open-meteo".to_string(),
                    payload: json!({ "temperature": temp, "lat": lat, "lon": lon }),
                    metadata: None,
                    entities: vec![],
                }]
            },
            Err(e) => {
                host::log_error(&format!("Weather: API call failed: {}", e)).await;
                vec![]
            }
        }
    }
}

scry_plugin!(WeatherPlugin);
