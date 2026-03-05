use scry_plugin_sdk::prelude::*;
use scry_plugin_sdk::schema::{namespaces, traits};

#[derive(Default)]
struct MusicEnricher;

impl ScryPlugin for MusicEnricher {
    fn get_manifest(&self) -> scry_plugin_sdk::Manifest {
        scry_plugin_sdk::Manifest {
            id: "scry-music-enricher".to_string(),
            name: "Music Visual Enricher".to_string(),
            version: "0.2.0".to_string(),
            description: "Reichert Musik-Entitäten (Artists, Alben) mit Bildern an, basierend auf deren Namen.".to_string(),
            subscriptions: vec!["music.scrobble".to_string()],
            capabilities: vec!["network".to_string(), "state".to_string()],
            exports: vec![],
            domain_info: vec![],
            predicates: vec![],
            provided_traits: vec![scry_plugin_sdk::TraitCapability {
                entity_namespace: namespaces::MUSIC.to_string(),
                entity_type: "artist".to_string(),
                trait_id: traits::PHOTO.to_string(),
            }],
            poll_interval: None,
            config_schema: None,
            suggested_widgets: vec![],
            oauth_config: None,
        }
    }

    fn on_ingest(&self, mut ev: SdkEvent) -> Result<SdkEvent, String> {
        // Dynamic confidence based on data quality
        if ev.category == "music.scrobble" {
            let has_track = ev.payload.get("track").is_some();
            ev.confidence = Some(if has_track { 1.0 } else { 0.6 });
        }
        Ok(ev)
    }

    fn resolve_trait(
        &self,
        namespace: &str,
        typ: &str,
        id: &str,
        trait_id: &str,
    ) -> Result<Option<String>, String> {
        if namespace == namespaces::MUSIC && typ == "artist" && trait_id == traits::PHOTO {
            // Get the display name from the host (don't use the ID, which might be a UUID)
            let name = host::get_entity_trait(namespace, typ, id, traits::NAME)
                .unwrap_or_else(|| id.to_string());
                
            let mock_url = format!(
                "https://ui-avatars.com/api/?name={}&background=random&size=128",
                urlencoding::encode(&name)
            );
            return Ok(Some(json!(mock_url).to_string()));
        }
        Ok(None)
    }

    fn on_entity_discovered(&self, namespace: &str, typ: &str, id: &str) {
        if namespace == namespaces::MUSIC && (typ == "artist" || typ == "album") {
            host::log_info(&format!(
                "Enricher: New music entity discovered: {} ({}). Fetching photo...",
                typ, id
            ));

            if let Ok(Some(photo_url_json)) =
                self.resolve_trait(namespace, typ, id, traits::PHOTO)
            {
                host::set_entity_trait(namespace, typ, id, traits::PHOTO, &photo_url_json);
            }
        }
    }
}

scry_plugin!(MusicEnricher);
