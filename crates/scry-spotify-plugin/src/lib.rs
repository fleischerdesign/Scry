use base64::Engine;
use scry_plugin_sdk::prelude::*;
use scry_plugin_sdk::schema::{namespaces, traits, predicates};
use serde::Deserialize;

#[derive(Default)]
struct SpotifyPlugin;

#[derive(Debug, Deserialize)]
struct SpotifyTokenResponse {
    access_token: Option<String>,
    expires_in: Option<u32>,
    refresh_token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SpotifyCurrentlyPlaying {
    item: Option<SpotifyTrack>,
    is_playing: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct SpotifyTrack {
    id: Option<String>,
    name: Option<String>,
    artists: Option<Vec<SpotifyArtist>>,
    album: Option<SpotifyAlbum>,
    duration_ms: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct SpotifyArtist {
    id: Option<String>,
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SpotifyAlbum {
    id: Option<String>,
    name: Option<String>,
    images: Option<Vec<SpotifyImage>>,
}

#[derive(Debug, Deserialize)]
struct SpotifyImage {
    url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SpotifyRecentTracks {
    items: Vec<SpotifyRecentItem>,
}

#[derive(Debug, Deserialize)]
struct SpotifyRecentItem {
    played_at: Option<String>,
    track: Option<SpotifyTrack>,
}

#[derive(Debug, Deserialize)]
struct SpotifyArtistDetail {
    images: Option<Vec<SpotifyImage>>,
}

impl ScryPlugin for SpotifyPlugin {
    fn get_manifest(&self) -> scry_plugin_sdk::Manifest {
        scry_plugin_sdk::Manifest {
            id: "scry-spotify-plugin".to_string(),
            name: "Spotify".to_string(),
            version: "0.2.1".to_string(),
            description:
                "Importiert Spotify Playback History mit deterministischen UUIDs für Musik-Entitäten."
                    .to_string(),
            subscriptions: vec!["spotify.playback".to_string()],
            capabilities: vec![
                "network".to_string(),
                "state".to_string(),
                "config".to_string(),
                "oauth".to_string(),
            ],
            exports: vec![
                scry_plugin_sdk::DataField {
                    category: "spotify.playback".to_string(),
                    path: "track_name".to_string(),
                    semantic_type: "entity.music.track".to_string(),
                    description: "Name des gespielten Tracks".to_string(),
                    format: None,
                    icon: Some("lucide:music".to_string()),
                    unit: None,
                    privacy: Some("pii".to_string()),
                    confidence: Some(1.0),
                    temporal: None,
                },
                scry_plugin_sdk::DataField {
                    category: "spotify.playback".to_string(),
                    path: "artist_names".to_string(),
                    semantic_type: "entity.music.artist".to_string(),
                    description: "Namen aller beteiligten Künstler".to_string(),
                    format: Some("json-array".to_string()),
                    icon: Some("lucide:mic".to_string()),
                    unit: None,
                    privacy: Some("pii".to_string()),
                    confidence: Some(1.0),
                    temporal: None,
                },
                scry_plugin_sdk::DataField {
                    category: "spotify.playback".to_string(),
                    path: "album_name".to_string(),
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
                    category: "spotify.status".to_string(),
                    path: "is_playing".to_string(),
                    semantic_type: "boolean.music.playing".to_string(),
                    description: "Ob aktuell Musik gespielt wird".to_string(),
                    format: None,
                    icon: Some("lucide:play-circle".to_string()),
                    unit: None,
                    privacy: None,
                    confidence: Some(1.0),
                    temporal: Some("absolute".to_string()),
                },
            ],
            domain_info: vec![scry_plugin_sdk::DomainInfo {
                ns: namespaces::MUSIC.to_string(),
                icon: Some("lucide:headphones".to_string()),
            }],
            predicates: vec![
                scry_plugin_sdk::PredicateDefinition {
                    id: predicates::PLAYED_BY.to_string(),
                    label: "Played by".to_string(),
                    inverse_label: "Plays".to_string(),
                },
                scry_plugin_sdk::PredicateDefinition {
                    id: predicates::ON_ALBUM.to_string(),
                    label: "On Album".to_string(),
                    inverse_label: "Contains Track".to_string(),
                },
            ],
            provided_traits: vec![scry_plugin_sdk::TraitCapability {
                entity_namespace: namespaces::MUSIC.to_string(),
                entity_type: "artist".to_string(),
                trait_id: traits::PHOTO.to_string(),
            }],
            poll_interval: Some(60),
            config_schema: Some(
                json!({
                    "type": "object",
                    "properties": {
                        "client_id": {
                            "type": "string",
                            "description": "Spotify App Client ID (aus developer.spotify.com)",
                            "secret": true
                        },
                        "client_secret": {
                            "type": "string",
                            "description": "Spotify App Client Secret",
                            "secret": true
                        }
                    }
                })
                .to_string(),
            ),
            oauth_config: Some(scry_plugin_sdk::OAuthConfig {
                auth_url: "https://accounts.spotify.com/authorize".to_string(),
                token_url: "https://accounts.spotify.com/api/token".to_string(),
                scopes: vec![
                    "user-read-recently-played".to_string(),
                    "user-read-currently-playing".to_string(),
                    "user-read-playback-state".to_string(),
                ],
            }),
            suggested_widgets: vec![],
        }
    }

    fn on_init(&self) -> Result<(), String> {
        host::log_info("Spotify Plugin (v5 ID) initialized");
        Ok(())
    }

    fn resolve_trait(
        &self,
        namespace: &str,
        typ: &str,
        id: &str,
        trait_id: &str,
    ) -> Result<Option<String>, String> {
        if namespace == namespaces::MUSIC && typ == "artist" && trait_id == traits::PHOTO {
            let client_id = match host::get_secret("client_id") {
                Some(id) => id,
                None => return Ok(None),
            };
            let client_secret = match host::get_secret("client_secret") {
                Some(secret) => secret,
                None => return Ok(None),
            };

            let artist_id_spotify = match host::get_entity_trait(namespace, typ, id, "scry.spotify/artist_id") {
                Some(sid_json) => serde_json::from_str::<String>(&sid_json).unwrap_or_default(),
                None => return Ok(None),
            };

            if artist_id_spotify.is_empty() { return Ok(None); }

            let token = match self.get_valid_access_token(&client_id, &client_secret) {
                Some(t) => t,
                None => return Ok(None),
            };

            let url = format!("https://api.spotify.com/v1/artists/{}", artist_id_spotify);
            let headers = vec![("Authorization".to_string(), format!("Bearer {}", token))];

            match host::http_request("GET", &url, None, headers) {
                Ok(resp) if resp.status == 200 => {
                    let detail: SpotifyArtistDetail = serde_json::from_str(&resp.body).map_err(|e| e.to_string())?;
                    let photo_url = detail.images.and_then(|imgs| imgs.first().map(|i| i.url.clone()));
                    return Ok(photo_url.flatten().map(|u| json!(u).to_string()));
                }
                _ => return Ok(None),
            }
        }
        Ok(None)
    }

    fn on_entity_discovered(&self, namespace: &str, typ: &str, id: &str) {
        if namespace == namespaces::MUSIC && typ == "artist" {
            host::log_info(&format!("Spotify: New artist discovered: {}. Resolving photo...", id));
            if let Ok(Some(photo_url_json)) = self.resolve_trait(namespace, typ, id, traits::PHOTO) {
                host::set_entity_trait(namespace, typ, id, traits::PHOTO, &photo_url_json);
            }
        }
    }

    fn on_poll(&self) -> Vec<SdkEvent> {
        host::log_info("Spotify: on_poll started");
        
        let client_id = match host::get_secret("client_id") {
            Some(id) => id,
            None => { host::log_warn("Spotify: No client_id configured"); return vec![]; }
        };

        let client_secret = match host::get_secret("client_secret") {
            Some(secret) => secret,
            None => { host::log_warn("Spotify: No client_secret configured"); return vec![]; }
        };

        // 1. Intelligent Token Management
        let access_token = self.get_valid_access_token(&client_id, &client_secret);
        let token = match access_token {
            Some(t) => t,
            None => { host::log_warn("Spotify: Could not obtain access token"); return vec![]; }
        };

        // 2. Fetch History (less frequent or delta-based)
        // We only fetch history if it's been a while (e.g., every 5th poll) to save resources
        let poll_count = host::get_state("internal_poll_count").and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
        host::set_state("internal_poll_count", &(poll_count + 1).to_string());

        let mut all_events = Vec::new();
        if poll_count % 5 == 0 {
            host::log_info("Spotify: Fetching recent tracks (History Sync)");
            let recent_events = self.fetch_recent_tracks(&token);
            all_events.extend(recent_events);
        }

        // 3. Fetch Currently Playing (Every poll, but with delta-check)
        let playing_events = self.fetch_currently_playing(&token);
        all_events.extend(playing_events);

        all_events
    }

    fn on_ingest(&self, mut ev: SdkEvent) -> Result<SdkEvent, String> {
        if ev.category.starts_with("spotify.") {
            // --- Common Data extraction ---
            let track_name = ev.payload.get("track_name").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string();
            let artist_names: Vec<String> = ev.payload.get("artist_names").and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();
            let primary_artist = artist_names.first().map(|s| s.as_str()).unwrap_or("Unknown");
            let album_name = ev.payload.get("album_name").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string();
            let album_image = ev.payload.get("album_image").and_then(|v| v.as_str());

            // --- Display fields ---
            ev.display_title = Some(track_name.clone());
            ev.display_subtitle = Some(format!("by {}", artist_names.join(", ")));

            // --- Track Entity (UUID v5) ---
            let track_id = identity::create_id(namespaces::MUSIC, &["track", primary_artist, &track_name]);
            ev.entities.push(scry_plugin_sdk::EntityRef {
                path: "payload.track_name".to_string(),
                namespace: namespaces::MUSIC.to_string(),
                typ: "track".to_string(),
                id: track_id.clone(),
            });
            host::set_entity_trait(namespaces::MUSIC, "track", &track_id, traits::NAME, &json!(track_name).to_string());
            host::set_entity_trait(namespaces::MUSIC, "track", &track_id, traits::SUBTITLE, &json!(artist_names.join(", ")).to_string());
            host::set_entity_trait(namespaces::MUSIC, "track", &track_id, traits::ICON, &json!("lucide:music").to_string());
            if let Some(img) = album_image {
                host::set_entity_trait(namespaces::MUSIC, "track", &track_id, traits::PHOTO, &json!(img).to_string());
            }
            if let Some(track_id_spotify) = ev.payload.get("track_id").and_then(|v| v.as_str()) {
                 host::set_entity_trait(namespaces::MUSIC, "track", &track_id, "scry.spotify/track_id", &json!(track_id_spotify).to_string());
                 
                 // Add agnostic external link
                 let spotify_link = json!([{
                     "label": "Spotify",
                     "url": format!("https://open.spotify.com/track/{}", track_id_spotify),
                     "icon": "lucide:external-link"
                 }]);
                 host::set_entity_trait(namespaces::MUSIC, "track", &track_id, traits::LINKS, &spotify_link.to_string());
            }

            // --- Artist Entities ---
            let artist_ids_spotify: Vec<String> = ev.payload.get("artist_ids").and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();

            for (i, artist) in artist_names.iter().enumerate() {
                let artist_id = identity::create_id(namespaces::MUSIC, &["artist", artist]);
                ev.entities.push(scry_plugin_sdk::EntityRef {
                    path: "payload.artist_names".to_string(),
                    namespace: namespaces::MUSIC.to_string(),
                    typ: "artist".to_string(),
                    id: artist_id.clone(),
                });
                host::set_entity_trait(namespaces::MUSIC, "artist", &artist_id, traits::NAME, &json!(artist).to_string());
                host::set_entity_trait(namespaces::MUSIC, "artist", &artist_id, traits::ICON, &json!("lucide:mic").to_string());
                
                if let Some(sid) = artist_ids_spotify.get(i) {
                    host::set_entity_trait(namespaces::MUSIC, "artist", &artist_id, "scry.spotify/artist_id", &json!(sid).to_string());
                    
                    // Add agnostic external link
                    let spotify_link = json!([{
                        "label": "Spotify",
                        "url": format!("https://open.spotify.com/artist/{}", sid),
                        "icon": "lucide:external-link"
                    }]);
                    host::set_entity_trait(namespaces::MUSIC, "artist", &artist_id, traits::LINKS, &spotify_link.to_string());
                }

                // Track → played_by → Artist relationship
                host::set_relationship(scry_plugin_sdk::Relationship {
                    source_namespace: namespaces::MUSIC.to_string(),
                    source_type: "track".to_string(),
                    source_id: track_id.clone(),
                    predicate: predicates::PLAYED_BY.to_string(),
                    target_namespace: namespaces::MUSIC.to_string(),
                    target_type: "artist".to_string(),
                    target_id: artist_id.clone(),
                });
            }

            // --- Album Entity ---
            let album_id = identity::create_id(namespaces::MUSIC, &["album", primary_artist, &album_name]);
            ev.entities.push(scry_plugin_sdk::EntityRef {
                path: "payload.album_name".to_string(),
                namespace: namespaces::MUSIC.to_string(),
                typ: "album".to_string(),
                id: album_id.clone(),
            });
            host::set_entity_trait(namespaces::MUSIC, "album", &album_id, traits::NAME, &json!(album_name).to_string());
            host::set_entity_trait(namespaces::MUSIC, "album", &album_id, traits::ICON, &json!("lucide:disc").to_string());
            if let Some(img) = album_image {
                host::set_entity_trait(namespaces::MUSIC, "album", &album_id, traits::PHOTO, &json!(img).to_string());
            }
            if let Some(album_id_spotify) = ev.payload.get("album_id").and_then(|v| v.as_str()) {
                // Add agnostic external link
                let spotify_link = json!([{
                    "label": "Spotify",
                    "url": format!("https://open.spotify.com/album/{}", album_id_spotify),
                    "icon": "lucide:external-link"
                }]);
                host::set_entity_trait(namespaces::MUSIC, "album", &album_id, traits::LINKS, &spotify_link.to_string());
            }
            
            // Track → belongs_to → Album relationship
            host::set_relationship(scry_plugin_sdk::Relationship {
                source_namespace: namespaces::MUSIC.to_string(),
                source_type: "track".to_string(),
                source_id: track_id.clone(),
                predicate: predicates::ON_ALBUM.to_string(),
                target_namespace: namespaces::MUSIC.to_string(),
                target_type: "album".to_string(),
                target_id: album_id.clone(),
            });
        }
        Ok(ev)
    }
}

impl SpotifyPlugin {
    /// Extracts all artist names and IDs from a SpotifyTrack.
    /// Returns (names, ids) — both guaranteed to have the same length.
    fn extract_artists(track: &SpotifyTrack) -> (Vec<String>, Vec<String>) {
        match &track.artists {
            Some(artists) if !artists.is_empty() => {
                let names = artists
                    .iter()
                    .map(|a| {
                        a.name
                            .clone()
                            .unwrap_or_else(|| "Unknown Artist".to_string())
                    })
                    .collect();
                let ids = artists
                    .iter()
                    .map(|a| a.id.clone().unwrap_or_default())
                    .collect();
                (names, ids)
            }
            _ => (vec!["Unknown Artist".to_string()], vec![String::new()]),
        }
    }

    /// Builds the payload JSON for a track, shared between fetch_recent_tracks and fetch_currently_playing.
    fn build_track_payload(
        track: &SpotifyTrack,
        artist_names: &[String],
        artist_ids: &[String],
    ) -> serde_json::Value {
        let track_name = track.name.clone().unwrap_or_else(|| "Unknown".to_string());
        let album_name = track
            .album
            .as_ref()
            .and_then(|a| a.name.clone())
            .unwrap_or_else(|| "Unknown Album".to_string());
        let album_image = track
            .album
            .as_ref()
            .and_then(|a| a.images.as_ref())
            .and_then(|imgs| imgs.first())
            .and_then(|i| i.url.clone());

        json!({
            "track_name": track_name,
            "artist_names": artist_names,
            "artist_ids": artist_ids,
            // Flattened convenience field for display / backward compat
            "artist_name": artist_names.join(", "),
            "album_name": album_name,
            "album_image": album_image,
            "track_id": track.id.clone().unwrap_or_default(),
            "album_id": track.album.as_ref().and_then(|a| a.id.clone()).unwrap_or_default(),
            "duration_ms": track.duration_ms.unwrap_or(0),
        })
    }

    fn refresh_access_token(
        &self,
        client_id: &str,
        client_secret: &str,
        refresh_token: &str,
    ) -> Option<String> {
        let body = format!(
            "grant_type=refresh_token&refresh_token={}",
            urlencoding::encode(refresh_token)
        );

        let credentials = base64::engine::general_purpose::STANDARD
            .encode(format!("{}:{}", client_id, client_secret));

        let headers = vec![
            (
                "Authorization".to_string(),
                format!("Basic {}", credentials),
            ),
            (
                "Content-Type".to_string(),
                "application/x-www-form-urlencoded".to_string(),
            ),
        ];

        match host::http_post(
            "https://accounts.spotify.com/api/token",
            Some(body),
            headers,
        ) {
            Ok(resp_body) => {
                let token_response: SpotifyTokenResponse = match serde_json::from_str(&resp_body) {
                    Ok(tr) => tr,
                    Err(e) => {
                        host::log_error(&format!(
                            "Spotify: Failed to parse token response: {}. Body: {}",
                            e, resp_body
                        ));
                        return None;
                    }
                };
                if let Some(new_refresh) = &token_response.refresh_token {
                    host::set_state("oauth_refresh_token", new_refresh);
                }
                token_response.access_token
            }
            Err(e) => {
                host::log_error(&format!("Spotify: Failed to refresh token: {}", e));
                None
            }
        }
    }

    fn fetch_recent_tracks(&self, access_token: &str) -> Vec<SdkEvent> {
        let headers = vec![(
            "Authorization".to_string(),
            format!("Bearer {}", access_token),
        )];

        let last_played_at = host::get_state("last_played_at");
        let mut url = "https://api.spotify.com/v1/me/player/recently-played?limit=50".to_string();
        if let Some(ts) = &last_played_at {
            url = format!("{}&after={}", url, ts);
        }

        let response = match host::http_request(
            "GET",
            &url,
            None,
            headers,
        ) {
            Ok(res) => res,
            Err(e) => {
                host::log_error(&format!("Spotify API error: {}", e));
                return vec![];
            }
        };

        if response.status != 200 {
            host::log_error(&format!(
                "Spotify API returned status {}: {}",
                response.status, response.body
            ));
            return vec![];
        }

        let recent: SpotifyRecentTracks = match serde_json::from_str(&response.body) {
            Ok(t) => t,
            Err(e) => {
                host::log_error(&format!("Failed to parse Spotify response: {}", e));
                return vec![];
            }
        };

        let mut events = Vec::new();
        let mut max_timestamp_ms: i64 = last_played_at
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(0);

        for item in recent.items {
            if let Some(track) = item.track {
                let played_at = item
                    .played_at
                    .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());

                let dt = chrono::DateTime::parse_from_rfc3339(&played_at)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .unwrap_or_else(|_| chrono::Utc::now());

                let ts_ms = dt.timestamp_millis();
                if ts_ms > max_timestamp_ms {
                    max_timestamp_ms = ts_ms;
                }

                let (artist_names, artist_ids) = Self::extract_artists(&track);
                let payload = Self::build_track_payload(&track, &artist_names, &artist_ids);
                let display_image = track
                    .album
                    .as_ref()
                    .and_then(|a| a.images.as_ref())
                    .and_then(|imgs| imgs.first())
                    .and_then(|i| i.url.clone());
                
                let track_name = track.name.clone().unwrap_or_else(|| "Unknown".to_string());
                let subtitle = format!("by {}", artist_names.join(", "));

                let mut ev = SdkEvent::new("spotify.playback", "spotify", payload)
                    .with_context("alias:self")
                    .with_title(track_name)
                    .with_subtitle(subtitle)
                    .with_confidence(1.0);

                ev.timestamp = dt;
                if let Some(img) = display_image {
                    ev = ev.with_image(img);
                }

                events.push(ev);
            }
        }

        if max_timestamp_ms > 0 {
            host::set_state("last_played_at", &max_timestamp_ms.to_string());
        }

        events
    }

    fn fetch_currently_playing(&self, access_token: &str) -> Vec<SdkEvent> {
        let headers = vec![(
            "Authorization".to_string(),
            format!("Bearer {}", access_token),
        )];

        let response = match host::http_request(
            "GET",
            "https://api.spotify.com/v1/me/player/currently-playing",
            None,
            headers,
        ) {
            Ok(res) => res,
            Err(_) => return vec![],
        };

        if response.status == 204 {
            let last_id = host::get_state("last_playing_id").unwrap_or_default();
            if !last_id.is_empty() {
                host::set_entity_trait(namespaces::CORE, "user", "self", traits::NOW_PLAYING, "null");
                host::set_state("last_playing_id", "");
            }
            return vec![];
        }

        if response.status != 200 {
            host::log_error(&format!("Spotify currently-playing API status: {}", response.status));
            return vec![];
        }

        let playing: SpotifyCurrentlyPlaying = match serde_json::from_str(&response.body) {
            Ok(p) => p,
            Err(e) => {
                host::log_error(&format!("Failed to parse currently-playing response: {}", e));
                return vec![];
            }
        };

        let is_playing = playing.is_playing.unwrap_or(false);

        if is_playing && playing.item.is_some() {
            let track = playing.item.unwrap();
            let track_id_spotify = track.id.clone().unwrap_or_default();
            
            // Delta Check: Only update if the track has actually changed
            let last_playing_id = host::get_state("last_playing_id").unwrap_or_default();
            if track_id_spotify == last_playing_id {
                return vec![];
            }
            host::set_state("last_playing_id", &track_id_spotify);

            let (artist_names, _artist_ids) = Self::extract_artists(&track);
            let track_name = track.name.clone().unwrap_or_default();
            
            // 1. Resolve the deterministic track ID
            let track_id = identity::create_id(namespaces::MUSIC, &["track", &artist_names[0], &track_name]);
            
            // 2. Proactively store metadata for this track entity so it's resolvable
            host::set_entity_trait(namespaces::MUSIC, "track", &track_id, traits::NAME, &json!(track_name).to_string());
            host::set_entity_trait(namespaces::MUSIC, "track", &track_id, traits::SUBTITLE, &json!(artist_names.join(", ")).to_string());
            if let Some(img) = track.album.as_ref().and_then(|a| a.images.as_ref()).and_then(|imgs| imgs.first()).and_then(|i| i.url.clone()) {
                host::set_entity_trait(namespaces::MUSIC, "track", &track_id, traits::PHOTO, &json!(img).to_string());
            }
            
            // Add agnostic external link for real-time focus
            let spotify_link = json!([{
                "label": "Spotify",
                "url": format!("https://open.spotify.com/track/{}", track_id_spotify),
                "icon": "lucide:external-link"
            }]);
            host::set_entity_trait(namespaces::MUSIC, "track", &track_id, traits::LINKS, &spotify_link.to_string());

            // 3. Set relationships and artist metadata immediately
            let (_, artist_ids_spotify) = Self::extract_artists(&track);
            for (i, artist_name) in artist_names.iter().enumerate() {
                let artist_id = identity::create_id(namespaces::MUSIC, &["artist", artist_name]);
                
                // Store artist metadata
                host::set_entity_trait(namespaces::MUSIC, "artist", &artist_id, traits::NAME, &json!(artist_name).to_string());
                host::set_entity_trait(namespaces::MUSIC, "artist", &artist_id, traits::ICON, &json!("lucide:mic").to_string());

                if let Some(sid) = artist_ids_spotify.get(i) {
                    if !sid.is_empty() {
                        host::set_entity_trait(namespaces::MUSIC, "artist", &artist_id, "scry.spotify/artist_id", &json!(sid).to_string());
                        
                        // Add agnostic external link
                        let spotify_link = json!([{
                            "label": "Spotify",
                            "url": format!("https://open.spotify.com/artist/{}", sid),
                            "icon": "lucide:external-link"
                        }]);
                        host::set_entity_trait(namespaces::MUSIC, "artist", &artist_id, traits::LINKS, &spotify_link.to_string());

                        // Proactively resolve photo if missing
                        if host::get_entity_trait(namespaces::MUSIC, "artist", &artist_id, traits::PHOTO).is_none() {
                            if let Ok(Some(photo_url_json)) = self.resolve_trait(namespaces::MUSIC, "artist", &artist_id, traits::PHOTO) {
                                host::set_entity_trait(namespaces::MUSIC, "artist", &artist_id, traits::PHOTO, &photo_url_json);
                            }
                        }
                    }
                }

                host::set_relationship(scry_plugin_sdk::Relationship {
                    source_namespace: namespaces::MUSIC.to_string(),
                    source_type: "track".to_string(),
                    source_id: track_id.clone(),
                    predicate: predicates::PLAYED_BY.to_string(),
                    target_namespace: namespaces::MUSIC.to_string(),
                    target_type: "artist".to_string(),
                    target_id: artist_id,
                });
            }

            // 4. Set the user's status as a pure entity reference (Agnostic Link)
            let ref_link = format!("{}:track:{}", namespaces::MUSIC, track_id);
            host::set_entity_trait(namespaces::CORE, "user", "self", traits::NOW_PLAYING, &json!(ref_link).to_string());
            
            host::log_info(&format!("Spotify: Now playing changed to {}", track_name));
        } else {
            let last_id = host::get_state("last_playing_id").unwrap_or_default();
            if !last_id.is_empty() {
                host::set_entity_trait(namespaces::CORE, "user", "self", traits::NOW_PLAYING, "null");
                host::set_state("last_playing_id", "");
            }
        }

        // Return NO events for the timeline to keep it clean.
        vec![]
    }

    fn get_valid_access_token(&self, client_id: &str, client_secret: &str) -> Option<String> {
        let cached_token = host::get_state("oauth_access_token");
        let expires_at_str = host::get_state("oauth_token_expires_at");
        
        let now = chrono::Utc::now().timestamp();
        let expires_at = expires_at_str.and_then(|s| s.parse::<i64>().ok()).unwrap_or(0);

        // If we have a token and it's valid for at least another 60 seconds, use it
        if let Some(token) = cached_token {
            if expires_at > now + 60 {
                return Some(token);
            }
        }

        // Otherwise, refresh
        host::log_info("Spotify: Access token expired or missing, refreshing...");
        let refresh_token = host::get_secret("oauth_refresh_token")?;
        
        let body = format!(
            "grant_type=refresh_token&refresh_token={}",
            urlencoding::encode(&refresh_token)
        );

        let credentials = base64::engine::general_purpose::STANDARD
            .encode(format!("{}:{}", client_id, client_secret));

        let headers = vec![
            ("Authorization".to_string(), format!("Basic {}", credentials)),
            ("Content-Type".to_string(), "application/x-www-form-urlencoded".to_string()),
        ];

        match host::http_post("https://accounts.spotify.com/api/token", Some(body), headers) {
            Ok(resp_body) => {
                let tr: SpotifyTokenResponse = match serde_json::from_str(&resp_body) {
                    Ok(res) => res,
                    Err(e) => {
                        host::log_error(&format!("Spotify: Failed to parse token response: {}", e));
                        return None;
                    }
                };
                if let Some(new_access) = tr.access_token {
                    let new_expires = now + (tr.expires_in.unwrap_or(3600) as i64);
                    host::set_state("oauth_access_token", &new_access);
                    host::set_state("oauth_token_expires_at", &new_expires.to_string());
                    if let Some(new_refresh) = tr.refresh_token {
                        host::set_state("oauth_refresh_token", &new_refresh);
                    }
                    return Some(new_access);
                }
                None
            }
            Err(e) => {
                host::log_error(&format!("Spotify: HTTP error refreshing token: {}", e));
                None
            }
        }
    }
}

scry_plugin!(SpotifyPlugin);
