use scry_plugin_sdk::prelude::*;
use serde::Deserialize;
use base64::Engine;

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
            description: "Importiert deine Spotify Playback History und aktuellen Wiedergabestatus.".to_string(),
            subscriptions: vec!["spotify.playback".to_string()],
            capabilities: vec!["network".to_string(), "state".to_string(), "config".to_string()],
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
                    path: "artist_name".to_string(),
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
            domain_info: vec![
                scry_plugin_sdk::DomainInfo { ns: "scry.spotify".to_string(), icon: Some("lucide:headphones".to_string()) }
            ],
            predicates: vec![
                scry_plugin_sdk::PredicateDefinition {
                    id: "scry.spotify/played_by".to_string(),
                    label: "Played by".to_string(),
                    inverse_label: "Plays".to_string()
                }
            ],
            provided_traits: vec![],
            poll_interval: Some(60),
            config_schema: Some(json!({
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
                    },
                    "refresh_token": {
                        "type": "string",
                        "description": "Spotify Refresh Token (wird automatisch gespeichert)",
                        "secret": true
                    }
                }
            }).to_string()),
            suggested_widgets: vec![],
        }
    }

    async fn on_init(&self) -> Result<(), String> {
        host::log_info("Spotify Plugin initialized").await;
        Ok(())
    }

    async fn on_poll(&self) -> Vec<SdkEvent> {
        let client_id = match host::get_secret("client_id").await {
            Some(id) => id,
            None => {
                host::log_warn("Spotify: No client_id configured").await;
                return vec![];
            }
        };

        let client_secret = match host::get_secret("client_secret").await {
            Some(secret) => secret,
            None => {
                host::log_warn("Spotify: No client_secret configured").await;
                return vec![];
            }
        };

        let refresh_token = host::get_secret("refresh_token").await;

        let access_token = match refresh_token {
            Some(token) => self.refresh_access_token(&client_id, &client_secret, &token).await,
            None => None,
        };

        let token = match access_token {
            Some(t) => t,
            None => {
                host::log_warn("Spotify: Could not obtain access token").await;
                return vec![];
            }
        };

        let recent_events = self.fetch_recent_tracks(&token).await;
        let playing_events = self.fetch_currently_playing(&token).await;

        let mut events = recent_events;
        events.extend(playing_events);
        events
    }

    async fn on_ingest(&self, mut ev: SdkEvent) -> Result<SdkEvent, String> {
        if ev.category.starts_with("spotify.") {
            if let Some(track_name) = ev.payload.get("track_name").and_then(|v| v.as_str()) {
                ev.display_title = Some(track_name.to_string());
            }
            if let Some(artist_name) = ev.payload.get("artist_name").and_then(|v| v.as_str()) {
                ev.display_subtitle = Some(format!("by {}", artist_name));
            }
            
            ev.entities.push(scry_plugin_sdk::EntityRef {
                path: "payload.track_name".to_string(),
                namespace: "scry.music".to_string(),
                typ: "track".to_string(),
                id: ev.payload.get("track_name").and_then(|v| v.as_str()).unwrap_or("unknown").to_string(),
            });

            if let Some(artist) = ev.payload.get("artist_name").and_then(|v| v.as_str()) {
                ev.entities.push(scry_plugin_sdk::EntityRef {
                    path: "payload.artist_name".to_string(),
                    namespace: "scry.music".to_string(),
                    typ: "artist".to_string(),
                    id: artist.to_string(),
                });
            }
        }
        Ok(ev)
    }
}

impl SpotifyPlugin {
    async fn refresh_access_token(&self, client_id: &str, client_secret: &str, refresh_token: &str) -> Option<String> {
        let params = [
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
            ("client_id", client_id),
        ];

        let credentials = base64::engine::general_purpose::STANDARD.encode(
            format!("{}:{}", client_id, client_secret)
        );

        let client = reqwest::Client::new();
        let response = client
            .post("https://accounts.spotify.com/api/token")
            .form(&params)
            .header("Authorization", format!("Basic {}", credentials))
            .send()
            .await
            .ok()?;

        let token_response: SpotifyTokenResponse = response.json().await.ok()?;
        
        if let Some(new_refresh) = token_response.refresh_token {
            host::set_state("refresh_token", &new_refresh).await;
        }

        token_response.access_token
    }

    async fn fetch_recent_tracks(&self, access_token: &str) -> Vec<SdkEvent> {
        let client = reqwest::Client::new();
        let response = match client
            .get("https://api.spotify.com/v1/me/player/recently-played?limit=10")
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                host::log_error(&format!("Spotify API error: {}", e)).await;
                return vec![];
            }
        };

        let recent: SpotifyRecentTracks = match response.json().await {
            Ok(t) => t,
            Err(e) => {
                host::log_error(&format!("Failed to parse Spotify response: {}", e)).await;
                return vec![];
            }
        };

        let mut events = Vec::new();
        for item in recent.items {
            if let Some(track) = item.track {
                let track_name = track.name.unwrap_or_else(|| "Unknown".to_string());
                let artist_name = track.artists
                    .as_ref()
                    .and_then(|a| a.first())
                    .and_then(|a| a.name.clone())
                    .unwrap_or_else(|| "Unknown Artist".to_string());
                let album_name = track.album
                    .as_ref()
                    .and_then(|a| a.name.clone())
                    .unwrap_or_else(|| "Unknown Album".to_string());

                let played_at = item.played_at.unwrap_or_else(|| chrono::Utc::now().to_rfc3339());

                events.push(SdkEvent {
                    id: uuid::Uuid::new_v4(),
                    timestamp: chrono::DateTime::parse_from_rfc3339(&played_at)
                        .map(|dt| dt.with_timezone(&chrono::Utc))
                        .unwrap_or_else(|_| chrono::Utc::now()),
                    category: "spotify.playback".to_string(),
                    source: "spotify".to_string(),
                    payload: json!({
                        "track_name": track_name,
                        "artist_name": artist_name,
                        "album_name": album_name,
                        "track_id": track.id.unwrap_or_default(),
                        "artist_id": track.artists.as_ref().and_then(|a| a.first()).and_then(|a| a.id.clone()).unwrap_or_default(),
                        "album_id": track.album.as_ref().and_then(|a| a.id.clone()).unwrap_or_default(),
                        "duration_ms": track.duration_ms.unwrap_or(0),
                    }),
                    metadata: None,
                    entities: vec![],
                    context: vec!["alias:self".to_string()],
                    context_info: None,
                    display_image: track.album.as_ref()
                        .and_then(|a| a.images.as_ref())
                        .and_then(|imgs| imgs.first())
                        .and_then(|i| i.url.clone()),
                    display_value: None,
                    display_title: None,
                    display_subtitle: None,
                    confidence: Some(1.0),
                });
            }
        }

        events
    }

    async fn fetch_currently_playing(&self, access_token: &str) -> Vec<SdkEvent> {
        let client = reqwest::Client::new();
        let response = match client
            .get("https://api.spotify.com/v1/me/player/currently-playing")
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await
        {
            Ok(r) => r,
            Err(_) => return vec![],
        };

        if !response.status().is_success() {
            return vec![];
        }

        let playing: SpotifyCurrentlyPlaying = match response.json().await {
            Ok(p) => p,
            Err(_) => return vec![],
        };

        let is_playing = playing.is_playing.unwrap_or(false);
        
        if !is_playing || playing.item.is_none() {
            return vec![];
        }

        let track = playing.item.unwrap();
        let track_name = track.name.unwrap_or_else(|| "Unknown".to_string());
        let artist_name = track.artists
            .as_ref()
            .and_then(|a| a.first())
            .and_then(|a| a.name.clone())
            .unwrap_or_else(|| "Unknown Artist".to_string());
        let album_name = track.album
            .as_ref()
            .and_then(|a| a.name.clone())
            .unwrap_or_else(|| "Unknown Album".to_string());

        vec![SdkEvent {
            id: uuid::Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
            category: "spotify.status".to_string(),
            source: "spotify".to_string(),
            payload: json!({
                "track_name": track_name,
                "artist_name": artist_name,
                "album_name": album_name,
                "is_playing": true,
                "track_id": track.id.unwrap_or_default(),
                "artist_id": track.artists.as_ref().and_then(|a| a.first()).and_then(|a| a.id.clone()).unwrap_or_default(),
                "duration_ms": track.duration_ms.unwrap_or(0),
            }),
            metadata: None,
            entities: vec![],
            context: vec!["alias:self".to_string()],
            context_info: None,
            display_image: track.album.as_ref()
                .and_then(|a| a.images.as_ref())
                .and_then(|imgs| imgs.first())
                .and_then(|i| i.url.clone()),
            display_value: None,
            display_title: None,
            display_subtitle: None,
            confidence: Some(1.0),
        }]
    }
}

scry_plugin!(SpotifyPlugin);
