use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use anyhow::{anyhow, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use rand::Rng;

const NONCE_SIZE: usize = 12;
const KEY_SIZE: usize = 32;

#[derive(Clone)]
pub struct SecretService {
    master_key: [u8; KEY_SIZE],
}

impl SecretService {
    pub fn new() -> Self {
        let key_str = std::env::var("SCRYPT_MASTER_KEY")
            .unwrap_or_else(|_| "default-dev-key-change-in-prod!".to_string());

        let mut master_key = [0u8; KEY_SIZE];
        let key_bytes = key_str.as_bytes();
        for (i, byte) in key_bytes.iter().enumerate() {
            master_key[i % KEY_SIZE] ^= *byte;
        }
        if key_bytes.len() < KEY_SIZE {
            for i in key_bytes.len()..KEY_SIZE {
                master_key[i] = key_bytes[i % key_bytes.len()];
            }
        }

        Self { master_key }
    }

    fn derive_key(&self, user_id: i64) -> [u8; KEY_SIZE] {
        use argon2::Argon2;
        let salt = format!("scry-secret-{}", user_id);
        let salt_bytes = salt.as_bytes();
        let mut output = [0u8; KEY_SIZE];
        Argon2::default()
            .hash_password_into(self.master_key.as_slice(), salt_bytes, &mut output)
            .expect("Key derivation failed");
        output
    }

    pub fn encrypt(&self, user_id: i64, plaintext: &str) -> Result<String> {
        let key = self.derive_key(user_id);
        let cipher =
            Aes256Gcm::new_from_slice(&key).map_err(|e| anyhow!("Cipher init failed: {:?}", e))?;

        let mut nonce_bytes = [0u8; NONCE_SIZE];
        rand::thread_rng().fill(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| anyhow!("Encryption failed: {:?}", e))?;

        let mut result = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);

        Ok(BASE64.encode(&result))
    }

    pub fn decrypt(&self, user_id: i64, encrypted: &str) -> Result<Option<String>> {
        let data = match BASE64.decode(encrypted) {
            Ok(d) => d,
            Err(_) => return Ok(None),
        };

        if data.len() < NONCE_SIZE {
            return Ok(None);
        }

        let (nonce_bytes, ciphertext) = data.split_at(NONCE_SIZE);
        let nonce = Nonce::from_slice(nonce_bytes);
        let key = self.derive_key(user_id);
        let cipher = match Aes256Gcm::new_from_slice(&key) {
            Ok(c) => c,
            Err(_) => return Ok(None),
        };

        match cipher.decrypt(nonce, ciphertext) {
            Ok(plaintext) => Ok(Some(
                String::from_utf8(plaintext).map_err(|e| anyhow!("UTF8 error: {}", e))?,
            )),
            Err(_) => Ok(None),
        }
    }
}

impl Default for SecretService {
    fn default() -> Self {
        Self::new()
    }
}
