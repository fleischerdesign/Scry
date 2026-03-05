use scry_plugin_sdk::prelude::*;

#[derive(Default)]
struct MusicEnricher;

impl ScryPlugin for MusicEnricher {
    fn get_manifest(&self) -> scry_plugin_sdk::Manifest {
        scry_plugin_sdk::Manifest {
            id: "scry-music-enricher".to_string(),
            name: "Music Visual Enricher".to_string(),
            version: "0.1.0".to_string(),
            description: "Anreichern von Künstlern mit Bildern und Metadaten.".to_string(),
            subscriptions: vec!["music.scrobble".to_string()],
            capabilities: vec!["network".to_string(), "state".to_string()],
            exports: vec![],
            domain_info: vec![],
            predicates: vec![],
            provided_traits: vec![scry_plugin_sdk::TraitCapability {
                entity_namespace: "scry.music".to_string(),
                entity_type: "artist".to_string(),
                trait_id: "scry.visual/photo".to_string(),
            }],
            poll_interval: None,
            config_schema: None,
            suggested_widgets: vec![],
            oauth_config: None,
        }
    }

    fn on_ingest(&self, mut ev: SdkEvent) -> Result<SdkEvent, String> {
        if ev.category == "music.scrobble" {
            // Demo for dynamic confidence:
            // If we have an artist but maybe the track is unknown, we are less confident.
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
        if namespace == "scry.music" && typ == "artist" && trait_id == "scry.visual/photo" {
            // Wir nutzen ui-avatars.com für echtes visuelles Feedback
            let mock_url = format!(
                "https://ui-avatars.com/api/?name={}&background=random&size=128",
                urlencoding::encode(id)
            );
            return Ok(Some(json!(mock_url).to_string()));
        }
        Ok(None)
    }

    fn on_entity_discovered(&self, namespace: &str, typ: &str, id: &str) {
        if namespace == "scry.music" && typ == "artist" {
            host::log_info(&format!(
                "Enricher: New artist discovered: {}. Fetching photo...",
                id
            ));

            // Trait auflösen
            if let Ok(Some(photo_url_json)) =
                self.resolve_trait(namespace, typ, id, "scry.visual/photo")
            {
                // Persistent im Host speichern (Push-Modell)
                host::set_entity_trait(namespace, typ, id, "scry.visual/photo", &photo_url_json);
                host::log_info(&format!("Enricher: Successfully stored photo for {}", id));
            }
        }
    }
}

scry_plugin!(MusicEnricher);
