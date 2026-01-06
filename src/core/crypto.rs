use crate::core::error::{DeepSceneError, Result};
use argon2::{Argon2, PasswordHasher, password_hash::SaltString};
use chacha20::ChaCha20;
use chacha20::cipher::{KeyIvInit, StreamCipher};
use rand::Rng;

pub struct CryptoEngine;

impl CryptoEngine {
    pub fn derive_key(password: &str, salt: &[u8; 16]) -> Result<[u8; 32]> {
        let argon2 = Argon2::default();
        let salt_string = SaltString::encode_b64(salt)
            .map_err(|e| DeepSceneError::Encryption(format!("Salt encoding failed: {}", e)))?;

        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt_string)
            .map_err(|e| DeepSceneError::Encryption(format!("Key derivation failed: {}", e)))?;

        let hash = password_hash
            .hash
            .ok_or_else(|| DeepSceneError::Encryption("Hash generation failed".to_string()))?;

        let bytes = hash.as_bytes();
        if bytes.len() < 32 {
            return Err(DeepSceneError::Encryption(
                "Insufficient hash length".to_string(),
            ));
        }

        let mut key = [0u8; 32];
        key.copy_from_slice(&bytes[..32]);
        Ok(key)
    }

    pub fn encrypt(data: &[u8], password: &str) -> Result<Vec<u8>> {
        if password.is_empty() {
            return Err(DeepSceneError::Validation(
                "Encryption password cannot be empty. Please provide a valid password".to_string(),
            ));
        }

        let mut rng = rand::thread_rng();
        let salt: [u8; 16] = rng.r#gen();
        let nonce: [u8; 12] = rng.r#gen();

        let key = Self::derive_key(password, &salt)?;

        let checksum = blake3::hash(data);
        let checksum_bytes = &checksum.as_bytes()[0..16];

        let mut data_with_checksum = Vec::new();
        data_with_checksum.extend_from_slice(checksum_bytes);
        data_with_checksum.extend_from_slice(data);

        let mut cipher = ChaCha20::new(&key.into(), &nonce.into());
        let mut encrypted = data_with_checksum;
        cipher.apply_keystream(&mut encrypted);

        let mut result = Vec::new();
        result.extend_from_slice(&salt);
        result.extend_from_slice(&nonce);
        result.extend_from_slice(&encrypted);

        Ok(result)
    }

    pub fn decrypt(data: &[u8], password: &str) -> Result<Vec<u8>> {
        if password.is_empty() {
            return Err(DeepSceneError::Validation(
                "Encryption password cannot be empty. Please provide a valid password".to_string(),
            ));
        }

        if data.len() < 16 + 12 + 16 {
            return Err(DeepSceneError::Encryption(
                "Corrupted encrypted data".to_string(),
            ));
        }

        let salt: [u8; 16] = data[0..16]
            .try_into()
            .map_err(|_| DeepSceneError::Encryption("Invalid salt".to_string()))?;

        let nonce: [u8; 12] = data[16..28]
            .try_into()
            .map_err(|_| DeepSceneError::Encryption("Invalid nonce".to_string()))?;

        let encrypted = &data[28..];

        let key = Self::derive_key(password, &salt)?;

        let mut cipher = ChaCha20::new(&key.into(), &nonce.into());
        let mut decrypted = encrypted.to_vec();
        cipher.apply_keystream(&mut decrypted);

        if decrypted.len() < 16 {
            return Err(DeepSceneError::Encryption(
                "Authentication failed".to_string(),
            ));
        }

        let stored_checksum = &decrypted[0..16];
        let actual_data = &decrypted[16..];

        let computed_checksum = blake3::hash(actual_data);
        let computed_checksum_bytes = &computed_checksum.as_bytes()[0..16];

        if stored_checksum != computed_checksum_bytes {
            return Err(DeepSceneError::Encryption(
                "Authentication failed".to_string(),
            ));
        }

        Ok(actual_data.to_vec())
    }
}
