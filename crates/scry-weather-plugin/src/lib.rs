use scry_plugin_sdk::prelude::*;
use scry_plugin_sdk::schema::{namespaces, traits};
use serde::Deserialize;

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
            version: "0.3.0".to_string(),
            description: "Abfrage von Wetterdaten basierend auf dem Standort mit deterministischen Ort-IDs.".to_string(),
            subscriptions: vec!["weather.*".to_string()],
            capabilities: vec!["network".to_string(), "state".to_string(), "config".to_string()],
            exports: vec![
                scry_plugin_sdk::DataField {
                    category: "weather.current".to_string(),
                    path: "temperature".to_string(),
                    semantic_type: "metric.environment.temperature".to_string(),
                    description: "Aktuelle Temperatur in Celsius".to_string(),
                    format: None,
                    icon: Some("lucide:thermometer".to_string()),
                    unit: Some("celsius".to_string()),
                    privacy: None,
                    confidence: Some(1.0),
                    temporal: Some("absolute".to_string()),
                }
            ],
            domain_info: vec![
                scry_plugin_sdk::DomainInfo { ns: "scry.weather".to_string(), icon: Some("lucide:cloud-sun".to_string()) }
            ],
            predicates: vec![],
            provided_traits: vec![],
            poll_interval: Some(600), // Alle 10 Minuten
            config_schema: Some(json!({
                "type": "object",
                "properties": {
                    "latitude": { "type": "number", "description": "Manuelle Latitude (optional)" },
                    "longitude": { "type": "number", "description": "Manuelle Longitude (optional)" }
                }
            }).to_string()),
            suggested_widgets: vec![
                scry_plugin_sdk::WidgetDefinition {
                    id: "weather-temp-now".to_string(),
                    title: "Temperature Now".to_string(),
                    template: scry_plugin_sdk::WidgetTemplate::Metric,
                    config_json: json!({ "semantic_type": "metric.environment.temperature", "unit": "celsius" }).to_string(),
                },
                scry_plugin_sdk::WidgetDefinition {
                    id: "weather-temp-trend".to_string(),
                    title: "Weather Trend".to_string(),
                    template: scry_plugin_sdk::WidgetTemplate::Trend,
                    config_json: json!({ "semantic_type": "metric.environment.temperature", "days": 7 }).to_string(),
                }
            ],
            oauth_config: None,
        }
    }

    fn on_poll(&self) -> Vec<SdkEvent> {
        host::log_info("Weather: Polling current conditions...");

        let mut lat = host::get_config("latitude").and_then(|v| v.parse::<f64>().ok());
        let mut lon = host::get_config("longitude").and_then(|v| v.parse::<f64>().ok());

        if lat.is_none() || lon.is_none() {
            if let Some(loc_json) =
                host::get_entity_trait(namespaces::CORE, "user", "self", "scry.geo/location")
            {
                if let Ok(loc) = serde_json::from_str::<GeoLocation>(&loc_json) {
                    lat = Some(loc.latitude);
                    lon = Some(loc.longitude);
                }
            }
        }

        let lat = lat.unwrap_or(52.52);
        let lon = lon.unwrap_or(13.41);

        let url = format!(
            "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current_weather=true",
            lat, lon
        );

        match host::http_get(&url) {
            Ok(resp_json) => {
                let v: serde_json::Value = serde_json::from_str(&resp_json).unwrap_or_default();
                let temp = v["current_weather"]["temperature"].as_f64().unwrap_or(0.0);

                let city = host::get_entity_trait(namespaces::CORE, "user", "self", traits::CITY)
                    .and_then(|v| serde_json::from_str::<String>(&v).ok())
                    .unwrap_or_else(|| "Current Location".to_string());

                vec![SdkEvent {
                    id: uuid::Uuid::new_v4(),
                    timestamp: chrono::Utc::now(),
                    category: "weather.current".to_string(),
                    source: "open-meteo".to_string(),
                    payload: json!({ "temperature": temp, "lat": lat, "lon": lon, "city": city }),
                    metadata: None,
                    entities: vec![],
                    context: vec!["alias:self".to_string()],
                    context_info: None,
                    display_image: None,
                    display_value: Some(format!("{}°C", temp)),
                    display_title: Some(format!("Temperature in {}", city)),
                    display_subtitle: Some(format!("Currently {}°C", temp)),
                    confidence: Some(1.0),
                }]
            }
            Err(e) => {
                host::log_error(&format!("Weather: API call failed: {}", e));
                vec![]
            }
        }
    }

    fn on_ingest(&self, mut ev: SdkEvent) -> Result<SdkEvent, String> {
        if ev.category == "weather.current" {
            if let Some(city) = ev.payload.get("city").and_then(|v| v.as_str()) {
                let place_id = identity::create_id(namespaces::PLACE, &["city", city]);
                ev.entities.push(scry_plugin_sdk::EntityRef {
                    path: "payload.city".to_string(),
                    namespace: namespaces::PLACE.to_string(),
                    typ: "city".to_string(),
                    id: place_id.clone(),
                });
                host::set_entity_trait(namespaces::PLACE, "city", &place_id, traits::NAME, &json!(city).to_string());
            }
        }
        Ok(ev)
    }
}

scry_plugin!(WeatherPlugin);
