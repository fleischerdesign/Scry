use scry_plugin_sdk::prelude::*;
use scry_plugin_sdk::schema::{namespaces, traits, predicates};
use serde::Deserialize;

#[derive(Default)]
struct GithubPlugin;

// --- API Models ---
#[derive(Debug, Deserialize)]
struct GithubUser {
    login: String,
    avatar_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GithubEvent {
    id: String,
    #[serde(rename = "type")]
    event_type: String,
    actor: serde_json::Value,
    repo: serde_json::Value,
    payload: serde_json::Value,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl ScryPlugin for GithubPlugin {
    fn get_manifest(&self) -> scry_plugin_sdk::Manifest {
        scry_plugin_sdk::Manifest {
            id: "github".to_string(),
            name: "GitHub".to_string(),
            version: "0.2.0".to_string(),
            description: "Synchronisiert GitHub Aktivitäten (Push, PR, Issues) mit ETag-Caching und Graph-Mapping.".to_string(),
            subscriptions: vec!["github.*".to_string()],
            capabilities: vec![
                "network".to_string(),
                "state".to_string(),
                "config".to_string(),
                "oauth".to_string(),
            ],
            exports: vec![
                scry_plugin_sdk::DataField {
                    category: "github.activity".to_string(),
                    path: "repo_name".to_string(),
                    semantic_type: "entity.software.repo".to_string(),
                    description: "Repository Name".to_string(),
                    format: None,
                    icon: Some("lucide:github".to_string()),
                    unit: None,
                    privacy: None,
                    confidence: Some(1.0),
                    temporal: None,
                }
            ],
            domain_info: vec![scry_plugin_sdk::DomainInfo {
                ns: namespaces::SOFTWARE.to_string(),
                icon: Some("lucide:code-2".to_string()),
            }],
            predicates: vec![],
            provided_traits: vec![],
            poll_interval: Some(300), // 5 Minuten
            config_schema: Some(
                serde_json::json!({
                    "type": "object",
                    "properties": {
                        "client_id": {
                            "type": "string",
                            "description": "GitHub OAuth App Client ID",
                            "secret": true
                        },
                        "client_secret": {
                            "type": "string",
                            "description": "GitHub OAuth App Client Secret",
                            "secret": true
                        }
                    }
                })
                .to_string(),
            ),
            suggested_widgets: vec![],
            oauth_config: Some(scry_plugin_sdk::OAuthConfig {
                auth_url: "https://github.com/login/oauth/authorize".to_string(),
                token_url: "https://github.com/login/oauth/access_token".to_string(),
                scopes: vec!["repo".to_string(), "user".to_string()],
            }),
        }
    }

    fn on_poll(&self) -> Vec<SdkEvent> {
        let token = match host::get_secret("oauth_access_token") {
            Some(t) => t,
            None => {
                host::log_warn("GitHub: No oauth_access_token available");
                return vec![];
            }
        };

        let username = match self.get_github_username(&token) {
            Some(u) => u,
            None => {
                host::log_error("GitHub: Could not resolve username");
                return vec![];
            }
        };

        self.fetch_events(&username, &token)
    }

    fn on_ingest(&self, mut ev: SdkEvent) -> Result<SdkEvent, String> {
        if !ev.category.starts_with("github.") { return Ok(ev); }

        let repo_data = &ev.payload["repo"];
        let actor_data = &ev.payload["actor"];
        
        let repo_id = self.ensure_repo_entity(repo_data);
        let user_id = self.ensure_user_entity(actor_data);

        ev.entities.push(scry_plugin_sdk::EntityRef {
            path: "payload.repo.name".to_string(),
            namespace: namespaces::SOFTWARE.to_string(),
            typ: "repo".to_string(),
            id: repo_id.clone(),
        });

        host::set_relationship(scry_plugin_sdk::Relationship {
            source_namespace: namespaces::SOFTWARE.to_string(),
            source_type: "repo".to_string(),
            source_id: repo_id,
            predicate: predicates::OWNED_BY.to_string(),
            target_namespace: namespaces::SOFTWARE.to_string(),
            target_type: "user".to_string(),
            target_id: user_id,
        });

        let event_type = ev.category.replace("github.", "");
        match event_type.as_str() {
            "PushEvent" => {
                let size = ev.payload["payload"]["size"].as_u64().unwrap_or(0);
                let msg = ev.payload["payload"]["commits"][0]["message"].as_str().unwrap_or("New commits");
                ev.display_title = Some(format!("Push ({} commits) to {}", size, ev.payload["repo"]["name"].as_str().unwrap_or("unknown")));
                ev.display_subtitle = Some(msg.to_string());
            },
            "PullRequestEvent" => {
                let action = ev.payload["payload"]["action"].as_str().unwrap_or("updated");
                let title = ev.payload["payload"]["pull_request"]["title"].as_str().unwrap_or("PR");
                ev.display_title = Some(format!("PR {} in {}", action, ev.payload["repo"]["name"].as_str().unwrap_or("unknown")));
                ev.display_subtitle = Some(title.to_string());
            },
            "IssuesEvent" => {
                let action = ev.payload["payload"]["action"].as_str().unwrap_or("updated");
                let title = ev.payload["payload"]["issue"]["title"].as_str().unwrap_or("Issue");
                ev.display_title = Some(format!("Issue {} in {}", action, ev.payload["repo"]["name"].as_str().unwrap_or("unknown")));
                ev.display_subtitle = Some(title.to_string());
            },
            "IssueCommentEvent" => {
                let action = ev.payload["payload"]["action"].as_str().unwrap_or("created");
                let body = ev.payload["payload"]["comment"]["body"].as_str().unwrap_or("Comment");
                ev.display_title = Some(format!("Issue comment {} in {}", action, ev.payload["repo"]["name"].as_str().unwrap_or("unknown")));
                ev.display_subtitle = Some(body.chars().take(100).collect::<String>());
            },
            "CreateEvent" => {
                let ref_type = ev.payload["payload"]["ref_type"].as_str().unwrap_or("entity");
                let ref_name = ev.payload["payload"]["ref"].as_str().unwrap_or("");
                ev.display_title = Some(format!("Created {} {} in {}", ref_type, ref_name, ev.payload["repo"]["name"].as_str().unwrap_or("unknown")));
            },
            _ => {
                ev.display_title = Some(format!("GitHub: {}", event_type));
            }
        }

        Ok(ev)
    }
}

impl GithubPlugin {
    fn get_github_username(&self, token: &str) -> Option<String> {
        if let Some(cached) = host::get_state("github_username") {
            return Some(cached);
        }

        let headers = vec![
            ("Authorization".to_string(), format!("token {}", token)),
            ("User-Agent".to_string(), "Scry-App".to_string()),
        ];

        if let Ok(resp) = host::http_request("GET", "https://api.github.com/user", None, headers) {
            if let Ok(user) = serde_json::from_str::<GithubUser>(&resp.body) {
                host::set_state("github_username", &user.login);
                return Some(user.login);
            }
        }
        None
    }

    fn fetch_events(&self, username: &str, token: &str) -> Vec<SdkEvent> {
        let mut headers = vec![
            ("Authorization".to_string(), format!("token {}", token)),
            ("User-Agent".to_string(), "Scry-App".to_string()),
        ];

        if let Some(etag) = host::get_state("github_events_etag") {
            headers.push(("If-None-Match".to_string(), etag));
        }

        // Removed /public to fetch both public AND private events (authorized by oauth token)
        let url = format!("https://api.github.com/users/{}/events", username);
        let resp = match host::http_request("GET", &url, None, headers) {
            Ok(r) => r,
            Err(e) => {
                host::log_error(&format!("GitHub: API Request failed: {}", e));
                return vec![];
            }
        };

        if resp.status == 304 {
            return vec![];
        }

        if resp.status != 200 {
            host::log_error(&format!("GitHub: API returned status {}: {}", resp.status, resp.body));
            return vec![];
        }

        if let Some(new_etag) = resp.headers.iter().find(|(k, _)| k.to_lowercase() == "etag").map(|(_, v)| v) {
            host::set_state("github_events_etag", new_etag);
        }

        let events: Vec<GithubEvent> = serde_json::from_str(&resp.body).unwrap_or_default();
        let last_id = host::get_state("github_last_event_id").unwrap_or_default();
        
        let mut sdk_events = Vec::new();
        let mut newest_id = last_id.clone();

        for (i, ge) in events.into_iter().enumerate() {
            if ge.id == last_id { break; }
            if i == 0 { newest_id = ge.id.clone(); }

            let mut ev = SdkEvent::new(format!("github.{}", ge.event_type), "github", serde_json::json!({
                "id": ge.id,
                "actor": ge.actor,
                "repo": ge.repo,
                "payload": ge.payload,
            }))
            .with_context("alias:self")
            .with_confidence(1.0);

            ev.timestamp = ge.created_at;
            sdk_events.push(ev);
        }

        host::set_state("github_last_event_id", &newest_id);
        sdk_events
    }

        fn ensure_repo_entity(&self, repo_data: &serde_json::Value) -> String {
            let name = repo_data["name"].as_str().unwrap_or("unknown");
            let id = identity::create_id(namespaces::SOFTWARE, &["repo", name]);
            host::set_entity_trait(namespaces::SOFTWARE, "repo", &id, traits::NAME, &serde_json::json!(name).to_string());
            host::set_entity_trait(namespaces::SOFTWARE, "repo", &id, traits::ICON, &serde_json::json!("lucide:code-2").to_string());
            id
        }
    
        fn ensure_user_entity(&self, actor_data: &serde_json::Value) -> String {
            let login = actor_data["login"].as_str().unwrap_or("unknown");
            let id = identity::create_id(namespaces::SOFTWARE, &["user", login]);
            
            host::set_entity_trait(namespaces::SOFTWARE, "user", &id, traits::NAME, &serde_json::json!(login).to_string());
            host::set_entity_trait(namespaces::SOFTWARE, "user", &id, traits::ICON, &serde_json::json!("lucide:user").to_string());
            if let Some(avatar) = actor_data["avatar_url"].as_str() {
                host::set_entity_trait(namespaces::SOFTWARE, "user", &id, traits::AVATAR, &serde_json::json!(avatar).to_string());
            }
            id
        }
    
}

scry_plugin!(GithubPlugin);
