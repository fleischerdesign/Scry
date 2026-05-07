use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use password_hash::{PasswordHash, SaltString, rand_core::OsRng};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::domain::*;
use crate::error::{Error, Result};
use crate::repository::{EntityRepository, ProfileRepository, UserRepository};

const DEV_JWT_SECRET: &str = "scry_development_secret_do_not_use_in_production_123456789";

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i64,       // User ID
    pub scopes: String, // Comma separated scopes
    pub exp: usize,     // Expiration time
}

#[derive(Clone)]
pub struct AuthService {
    db: SqlitePool,
    jwt_secret: String,
}

impl AuthService {
    pub fn new(db: SqlitePool) -> Self {
        let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| DEV_JWT_SECRET.to_string());
        Self { db, jwt_secret }
    }

    fn generate_jwt(&self, user_id: i64, scopes: &str) -> Result<String> {
        let expiration = chrono::Utc::now()
            .checked_add_signed(chrono::Duration::days(7)) // 1 week duration for now
            .expect("valid timestamp")
            .timestamp() as usize;

        let claims = Claims {
            sub: user_id,
            scopes: scopes.to_string(),
            exp: expiration,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .map_err(|e| {
            tracing::error!("JWT Encoding Error: {}", e);
            Error::Internal
        })
    }

    pub fn verify_jwt(&self, token: &str) -> Option<(i64, Vec<String>)> {
        let validation = Validation::new(Algorithm::HS256);
        decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_bytes()),
            &validation,
        )
        .ok()
        .map(|data| {
            let claims = data.claims;
            (
                claims.sub,
                claims.scopes.split(',').map(|s| s.to_string()).collect(),
            )
        })
    }

    pub async fn register(&self, req: RegisterRequest) -> Result<AuthResponse> {
        let user_repo = UserRepository::new(&self.db);

        // Hash password with Argon2
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(req.password.as_bytes(), &salt)
            .map_err(|_| Error::Internal)?
            .to_string();

        let user_id = user_repo.create_user(&req.username, &password_hash).await?;

        // Ensure the 'self' user entity exists in the graph
        let entity_repo = EntityRepository::new(&self.db, user_id);
        entity_repo
            .ensure_entity("scry.core", "user", "self")
            .await?;

        let api_key_val = Uuid::new_v4().to_string();
        user_repo
            .create_api_key(user_id, &api_key_val, "Default Key", "all")
            .await?;

        // Try to find an existing avatar for the user to populate display_image
        let display_image = entity_repo
            .get_trait("scry.core", "user", "self", "scry.core/avatar")
            .await
            .ok()
            .flatten();

        // Generate JWT for the session
        let token = self.generate_jwt(user_id, "all")?;

        Ok(AuthResponse {
            api_key: token, // We return JWT as the primary token for web clients
            user: User {
                id: user_id,
                username: req.username,
                display_image,
            },
        })
    }

    pub async fn login(&self, req: LoginRequest) -> Result<AuthResponse> {
        let user_repo = UserRepository::new(&self.db);
        let (user_id, username, stored_hash) = user_repo
            .find_by_username(&req.username)
            .await?
            .ok_or_else(|| Error::Auth("User not found".to_string()))?;

        // Verify password with Argon2
        let parsed_hash = PasswordHash::new(&stored_hash).map_err(|_| Error::Internal)?;

        if Argon2::default()
            .verify_password(req.password.as_bytes(), &parsed_hash)
            .is_err()
        {
            return Err(Error::Auth("Invalid password".to_string()));
        }

        // Self-healing: Ensure 'self' entity exists
        let entity_repo = EntityRepository::new(&self.db, user_id);
        entity_repo
            .ensure_entity("scry.core", "user", "self")
            .await?;

        // Use centralized DRY method
        let (_title, _subtitle, display_image, _icon) = entity_repo
            .get_display_info("scry.core", "user", "self")
            .await;

        // Get scopes from existing API key (or use 'all' default)
        let _api_key = user_repo.get_api_key_by_user(user_id).await?;

        // Generate fresh JWT
        let token = self.generate_jwt(user_id, "all")?;

        Ok(AuthResponse {
            api_key: token,
            user: User {
                id: user_id,
                username,
                display_image,
            },
        })
    }

    pub async fn get_profile(&self, user_id: i64) -> Result<serde_json::Value> {
        let profile_repo = ProfileRepository::new(&self.db, user_id);
        let entity_repo = EntityRepository::new(&self.db, user_id);

        // Self-Healing
        entity_repo
            .ensure_entity("scry.core", "user", "self")
            .await?;

        let rows = profile_repo.get_all().await?;
        let mut map = serde_json::Map::new();

        for (k, v) in rows {
            map.insert(k.clone(), json!(v));

            // Auto-Sync to Knowledge Graph
            let trait_id = format!("scry.core/{}", k);
            let value_json = json!(v).to_string();
            entity_repo
                .set_trait_if_not_exists(
                    "scry.core",
                    "user",
                    "self",
                    "core",
                    &trait_id,
                    &value_json,
                )
                .await?;
        }

        Ok(serde_json::Value::Object(map))
    }

    pub async fn update_profile(
        &self,
        user_id: i64,
        req: serde_json::Map<String, serde_json::Value>,
    ) -> Result<()> {
        let profile_repo = ProfileRepository::new(&self.db, user_id);
        let entity_repo = EntityRepository::new(&self.db, user_id);

        for (k, v) in req {
            let v_str = v.as_str().unwrap_or("").to_string();
            profile_repo.set(&k, &v_str).await?;

            // Update semantic graph
            let trait_id = format!("scry.core/{}", k);
            let value_json = serde_json::to_string(&v).unwrap_or_else(|_| "null".to_string());
            entity_repo
                .set_trait("scry.core", "user", "self", "core", &trait_id, &value_json)
                .await?;
        }
        Ok(())
    }

    pub async fn get_all_user_ids(&self) -> Result<Vec<i64>> {
        let user_repo = UserRepository::new(&self.db);
        user_repo.get_all_ids().await
    }

    pub async fn verify_api_key(&self, key: &str) -> Result<Option<(i64, Vec<String>)>> {
        let user_repo = UserRepository::new(&self.db);
        let auth = user_repo.find_by_api_key(key).await?;

        Ok(auth.map(|(user_id, scopes_str)| {
            (
                user_id,
                scopes_str.split(',').map(|s| s.to_string()).collect(),
            )
        }))
    }
}
