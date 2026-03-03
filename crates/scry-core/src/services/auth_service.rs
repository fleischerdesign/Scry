use sqlx::SqlitePool;
use uuid::Uuid;
use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use password_hash::{PasswordHash, SaltString, rand_core::OsRng};
use serde_json::json;

use crate::domain::*;
use crate::error::{Error, Result};
use crate::repository::{UserRepository, ProfileRepository, EntityRepository};

#[derive(Clone)]
pub struct AuthService {
    db: SqlitePool,
}

impl AuthService {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }

    pub async fn register(&self, req: RegisterRequest) -> Result<AuthResponse> {
        let user_repo = UserRepository::new(&self.db);
        let _entity_repo = EntityRepository::new(&self.db, 0); // Temporary user_id 0

        // Hash password with Argon2
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2.hash_password(req.password.as_bytes(), &salt)
            .map_err(|_| Error::Internal)?
            .to_string();

        let user_id = user_repo.create_user(&req.username, &password_hash).await?;
        
        // Ensure the 'self' user entity exists in the graph
        let entity_repo = EntityRepository::new(&self.db, user_id);
        entity_repo.ensure_entity("scry.core", "user", "self").await?;

        let api_key = Uuid::new_v4().to_string();
        user_repo.create_api_key(user_id, &api_key, "Default Key", "all").await?;

        Ok(AuthResponse { api_key, user: User { id: user_id, username: req.username } })
    }

    pub async fn login(&self, req: LoginRequest) -> Result<AuthResponse> {
        let user_repo = UserRepository::new(&self.db);
        let (user_id, username, stored_hash) = user_repo.find_by_username(&req.username).await?
            .ok_or_else(|| Error::Auth("User not found".to_string()))?;
        
        // Verify password with Argon2
        let parsed_hash = PasswordHash::new(&stored_hash)
            .map_err(|_| Error::Internal)?;
        
        if Argon2::default().verify_password(req.password.as_bytes(), &parsed_hash).is_err() {
            return Err(Error::Auth("Invalid password".to_string()));
        }

        // Self-healing: Ensure 'self' entity exists
        let entity_repo = EntityRepository::new(&self.db, user_id);
        entity_repo.ensure_entity("scry.core", "user", "self").await?;

        let api_key = user_repo.get_api_key_by_user(user_id).await?;
        Ok(AuthResponse { api_key, user: User { id: user_id, username } })
    }

    pub async fn get_profile(&self, user_id: i64) -> Result<serde_json::Value> {
        let profile_repo = ProfileRepository::new(&self.db, user_id);
        let entity_repo = EntityRepository::new(&self.db, user_id);

        // Self-Healing
        entity_repo.ensure_entity("scry.core", "user", "self").await?;

        let rows = profile_repo.get_all().await?;
        let mut map = serde_json::Map::new();
        
        for (k, v) in rows {
            map.insert(k.clone(), json!(v));
            
            // Auto-Sync to Knowledge Graph
            let trait_id = format!("scry.core/{}", k);
            let value_json = json!(v).to_string();
            entity_repo.set_trait_if_not_exists("scry.core", "user", "self", "core", &trait_id, &value_json).await?;
        }
        
        Ok(serde_json::Value::Object(map))
    }

    pub async fn update_profile(&self, user_id: i64, req: serde_json::Map<String, serde_json::Value>) -> Result<()> {
        let profile_repo = ProfileRepository::new(&self.db, user_id);
        let entity_repo = EntityRepository::new(&self.db, user_id);

        for (k, v) in req {
            let v_str = v.as_str().unwrap_or("").to_string();
            profile_repo.set(&k, &v_str).await?;

            // Update semantic graph
            let trait_id = format!("scry.core/{}", k);
            let value_json = serde_json::to_string(&v).unwrap_or_else(|_| "null".to_string());
            entity_repo.set_trait("scry.core", "user", "self", "core", &trait_id, &value_json).await?;
        }
        Ok(())
    }
}
