use base64::Engine;
use scry_plugin_sdk::prelude::*;
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

impl ScryPlugin for SpotifyPlugin {
    fn get_manifest(&self) -> scry_plugin_sdk::Manifest {
        scry_plugin_sdk::Manifest {
            id: "scry-spotify-plugin".to_string(),
            name: "Spotify".to_string(),
            version: "0.1.0".to_string(),
            description:
                "Importiert deine Spotify Playback History und aktuellen Wiedergabestatus."
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
                ns: "scry.spotify".to_string(),
                icon: Some("lucide:headphones".to_string()),
            }],
            predicates: vec![scry_plugin_sdk::PredicateDefinition {
                id: "scry.spotify/played_by".to_string(),
                label: "Played by".to_string(),
                inverse_label: "Plays".to_string(),
            }],
            provided_traits: vec![],
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
        host::log_info("Spotify Plugin initialized");
        Ok(())
    }

    fn on_poll(&self) -> Vec<SdkEvent> {
        host::log_info("Spotify: on_poll started");
        let client_id = match host::get_secret("client_id") {
            Some(id) => id,
            None => {
                host::log_warn("Spotify: No client_id configured");
                return vec![];
            }
        };

        let client_secret = match host::get_secret("client_secret") {
            Some(secret) => secret,
            None => {
                host::log_warn("Spotify: No client_secret configured");
                return vec![];
            }
        };

        let refresh_token = host::get_secret("oauth_refresh_token");

        let access_token = match refresh_token {
            Some(token) => {
                host::log_info("Spotify: Refreshing access token");
                self.refresh_access_token(&client_id, &client_secret, &token)
            }
            None => {
                host::log_warn("Spotify: No refresh token found");
                None
            }
        };

        let token = match access_token {
            Some(t) => t,
            None => {
                host::log_warn("Spotify: Could not obtain access token");
                return vec![];
            }
        };

        host::log_info("Spotify: Fetching recent tracks");
        let recent_events = self.fetch_recent_tracks(&token);
        host::log_info(&format!(
            "Spotify: Fetched {} recent tracks",
            recent_events.len()
        ));

        host::log_info("Spotify: Fetching currently playing");
        let playing_events = self.fetch_currently_playing(&token);

        let mut events = recent_events;
        events.extend(playing_events);

        events
    }

    fn on_ingest(&self, mut ev: SdkEvent) -> Result<SdkEvent, String> {
        if ev.category.starts_with("spotify.") {
            // --- Display fields ---
            if let Some(track_name) = ev.payload.get("track_name").and_then(|v| v.as_str()) {
                ev.display_title = Some(track_name.to_string());
            }
            // Use the array field for subtitle; fall back to flat string for backward compat
            let artist_label = ev
                .payload
                .get("artist_names")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .or_else(|| {
                    ev.payload
                        .get("artist_name")
                        .and_then(|v| v.as_str())
                        .map(String::from)
                });
            if let Some(label) = artist_label {
                ev.display_subtitle = Some(format!("by {}", label));
            }

            // --- Track entity ---
            let track_id = ev
                .payload
                .get("track_name")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();
            ev.entities.push(scry_plugin_sdk::EntityRef {
                path: "payload.track_name".to_string(),
                namespace: "scry.music".to_string(),
                typ: "track".to_string(),
                id: track_id.clone(),
            });

            // --- Artist entities (one per artist) ---
            let artist_names: Vec<String> = ev
                .payload
                .get("artist_names")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .or_else(|| {
                    ev.payload
                        .get("artist_name")
                        .and_then(|v| v.as_str())
                        .map(|s| vec![s.to_string()])
                })
                .unwrap_or_default();

            for artist in &artist_names {
                ev.entities.push(scry_plugin_sdk::EntityRef {
                    path: "payload.artist_names".to_string(),
                    namespace: "scry.music".to_string(),
                    typ: "artist".to_string(),
                    id: artist.clone(),
                });

                // Track → played_by → Artist relationship
                host::set_relationship(scry_plugin_sdk::Relationship {
                    source_namespace: "scry.music".to_string(),
                    source_type: "track".to_string(),
                    source_id: track_id.clone(),
                    predicate: "scry.spotify/played_by".to_string(),
                    target_namespace: "scry.music".to_string(),
                    target_type: "artist".to_string(),
                    target_id: artist.clone(),
                });
            }
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

        json!({
            "track_name": track_name,
            "artist_names": artist_names,
            "artist_ids": artist_ids,
            // Flattened convenience field for display / backward compat
            "artist_name": artist_names.join(", "),
            "album_name": album_name,
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

        let response = match host::http_request(
            "GET",
            "https://api.spotify.com/v1/me/player/recently-played?limit=10",
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

        host::log_info(&format!("Spotify: Raw response body: {}", response.body));

        let recent: SpotifyRecentTracks = match serde_json::from_str(&response.body) {
            Ok(t) => t,
            Err(e) => {
                host::log_error(&format!("Failed to parse Spotify response: {}", e));
                return vec![];
            }
        };

        let mut events = Vec::new();
        for item in recent.items {
            if let Some(track) = item.track {
                let (artist_names, artist_ids) = Self::extract_artists(&track);
                let payload = Self::build_track_payload(&track, &artist_names, &artist_ids);
                let display_image = track
                    .album
                    .as_ref()
                    .and_then(|a| a.images.as_ref())
                    .and_then(|imgs| imgs.first())
                    .and_then(|i| i.url.clone());

                let played_at = item
                    .played_at
                    .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());

                events.push(SdkEvent {
                    id: uuid::Uuid::new_v4(),
                    timestamp: chrono::DateTime::parse_from_rfc3339(&played_at)
                        .map(|dt| dt.with_timezone(&chrono::Utc))
                        .unwrap_or_else(|_| chrono::Utc::now()),
                    category: "spotify.playback".to_string(),
                    source: "spotify".to_string(),
                    payload,
                    metadata: None,
                    entities: vec![],
                    context: vec!["alias:self".to_string()],
                    context_info: None,
                    display_image,
                    display_value: None,
                    display_title: None,
                    display_subtitle: None,
                    confidence: Some(1.0),
                });
            }
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

        if response.status != 200 {
            host::log_error(&format!(
                "Spotify currently-playing API returned status: {}",
                response.status
            ));
            return vec![];
        }

        host::log_info(&format!(
            "Spotify: Raw currently-playing response: {}",
            response.body
        ));

        let playing: SpotifyCurrentlyPlaying = match serde_json::from_str(&response.body) {
            Ok(p) => p,
            Err(e) => {
                host::log_error(&format!(
                    "Failed to parse currently-playing response: {}",
                    e
                ));
                return vec![];
            }
        };

        let is_playing = playing.is_playing.unwrap_or(false);

        if !is_playing || playing.item.is_none() {
            return vec![];
        }

        let track = playing.item.unwrap();
        let (artist_names, artist_ids) = Self::extract_artists(&track);
        let mut payload = Self::build_track_payload(&track, &artist_names, &artist_ids);
        payload["is_playing"] = json!(true);
        let display_image = track
            .album
            .as_ref()
            .and_then(|a| a.images.as_ref())
            .and_then(|imgs| imgs.first())
            .and_then(|i| i.url.clone());

        vec![SdkEvent {
            id: uuid::Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
            category: "spotify.status".to_string(),
            source: "spotify".to_string(),
            payload,
            metadata: None,
            entities: vec![],
            context: vec!["alias:self".to_string()],
            context_info: None,
            display_image,
            display_value: None,
            display_title: None,
            display_subtitle: None,
            confidence: Some(1.0),
        }]
    }
}

scry_plugin!(SpotifyPlugin);
