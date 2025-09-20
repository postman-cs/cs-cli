//! Session Data Encryption for Cross-Device Sync
//!
//! Provides AES-256-GCM authenticated encryption for session cookies before
//! storing in GitHub gists. Master key stored in local keychain only.

use crate::{CsCliError, Result};
use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use hkdf::Hkdf;
use sha2::Sha256;
use std::process::Command;
use tracing::{debug, info};
use zeroize::Zeroize;

/// Salt for HKDF key derivation (application-specific)
const HKDF_SALT: &[u8] = b"cs-cli-session-encryption-v1.0";
const HKDF_INFO: &[u8] = b"github-gist-session-storage";
const KEYCHAIN_SERVICE: &str = "com.postman.cs-cli.session-key";
const KEYCHAIN_ACCOUNT: &str = "session-encryption-master-key";

pub struct SessionEncryption {
    cipher: Aes256Gcm,
}

impl Clone for SessionEncryption {
    fn clone(&self) -> Self {
        // Recreate the cipher with the same key derivation
        // This is safe because we're using the same keychain-stored master key
        Self::new().expect("Failed to clone SessionEncryption")
    }
}

impl SessionEncryption {
    /// Initialize encryption with keychain-stored master key
    pub fn new() -> Result<Self> {
        let master_key = get_or_create_master_key()?;

        // Derive encryption key using HKDF
        let hk = Hkdf::<Sha256>::new(Some(HKDF_SALT), &master_key);
        let mut derived_key = [0u8; 32]; // 256 bits for AES-256
        hk.expand(HKDF_INFO, &mut derived_key)
            .map_err(|_| CsCliError::Encryption("Key derivation failed".to_string()))?;

        let key = Key::<Aes256Gcm>::from_slice(&derived_key);
        let cipher = Aes256Gcm::new(key);

        // Clear derived key from memory
        derived_key.zeroize();

        Ok(Self { cipher })
    }

    /// Encrypt session data with authenticated encryption
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = self
            .cipher
            .encrypt(&nonce, plaintext)
            .map_err(|_| CsCliError::Encryption("Encryption failed".to_string()))?;

        // Prepend nonce to ciphertext (nonce + ciphertext + tag)
        let mut result = nonce.to_vec();
        result.extend_from_slice(&ciphertext);

        Ok(result)
    }

    /// Decrypt and authenticate session data
    pub fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>> {
        if ciphertext.len() < 12 {
            return Err(CsCliError::Encryption(
                "Invalid ciphertext length".to_string(),
            ));
        }

        // Extract nonce (first 12 bytes) and ciphertext
        let (nonce_bytes, encrypted_data) = ciphertext.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        let plaintext = self.cipher.decrypt(nonce, encrypted_data).map_err(|_| {
            CsCliError::Encryption("Decryption failed or data tampered".to_string())
        })?;

        Ok(plaintext)
    }
}

/// Get or create master encryption key in keychain
fn get_or_create_master_key() -> Result<Vec<u8>> {
    // Try to get existing key first
    match get_master_key_from_keychain() {
        Ok(key) => {
            debug!("Retrieved existing master key from keychain");
            Ok(key)
        }
        Err(_) => {
            info!("Generating new master encryption key...");
            let new_key = generate_master_key();
            store_master_key_in_keychain(&new_key)?;
            Ok(new_key)
        }
    }
}

/// Generate new random master key
fn generate_master_key() -> Vec<u8> {
    use rand::RngCore;
    let mut key = vec![0u8; 32]; // 256-bit key
    rand::thread_rng().fill_bytes(&mut key);
    key
}

/// Store master key securely in keychain
fn store_master_key_in_keychain(key: &[u8]) -> Result<()> {
    let key_b64 = base64_simd::STANDARD.encode_to_string(key);

    // Delete existing key (ignore errors)
    let _ = Command::new("security")
        .args([
            "delete-generic-password",
            "-s",
            KEYCHAIN_SERVICE,
            "-a",
            KEYCHAIN_ACCOUNT,
        ])
        .output();

    // Store new key
    let output = Command::new("security")
        .args([
            "add-generic-password",
            "-s",
            KEYCHAIN_SERVICE,
            "-a",
            KEYCHAIN_ACCOUNT,
            "-w",
            &key_b64,
            "-A", // Allow access for our app without user interaction
            "-T",
            &std::env::current_exe()
                .unwrap_or_default()
                .to_string_lossy(),
        ])
        .output()
        .map_err(|e| CsCliError::Encryption(format!("Failed to store master key: {e}")))?;

    if output.status.success() {
        info!("Master encryption key stored securely in keychain");
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(CsCliError::Encryption(format!(
            "Keychain storage failed: {stderr}"
        )))
    }
}

/// Retrieve master key from keychain
fn get_master_key_from_keychain() -> Result<Vec<u8>> {
    let output = Command::new("security")
        .args([
            "find-generic-password",
            "-s",
            KEYCHAIN_SERVICE,
            "-a",
            KEYCHAIN_ACCOUNT,
            "-w", // Return password only
        ])
        .output()
        .map_err(|e| CsCliError::Encryption(format!("Failed to access keychain: {e}")))?;

    if output.status.success() {
        let key_b64 = String::from_utf8(output.stdout)
            .map_err(|_| CsCliError::Encryption("Invalid keychain data".to_string()))?
            .trim()
            .to_string();

        let key = base64_simd::STANDARD
            .decode_to_vec(&key_b64)
            .map_err(|_| CsCliError::Encryption("Invalid base64 in keychain".to_string()))?;

        Ok(key)
    } else {
        Err(CsCliError::Encryption(
            "Master key not found in keychain".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_roundtrip() {
        let encryption = SessionEncryption::new().unwrap();
        let test_data = b"test session cookies data";

        let encrypted = encryption.encrypt(test_data).unwrap();
        let decrypted = encryption.decrypt(&encrypted).unwrap();

        assert_eq!(test_data, decrypted.as_slice());
    }

    #[test]
    fn test_tampered_data_fails() {
        let encryption = SessionEncryption::new().unwrap();
        let test_data = b"test data";

        let mut encrypted = encryption.encrypt(test_data).unwrap();
        // Tamper with the data
        let len = encrypted.len();
        encrypted[len - 1] ^= 1;

        // Should fail with authentication error
        assert!(encryption.decrypt(&encrypted).is_err());
    }

    #[test]
    fn test_different_nonces() {
        let encryption = SessionEncryption::new().unwrap();
        let test_data = b"same data";

        let encrypted1 = encryption.encrypt(test_data).unwrap();
        let encrypted2 = encryption.encrypt(test_data).unwrap();

        // Different nonces should produce different ciphertexts
        assert_ne!(encrypted1, encrypted2);

        // But both should decrypt to the same plaintext
        assert_eq!(encryption.decrypt(&encrypted1).unwrap(), test_data);
        assert_eq!(encryption.decrypt(&encrypted2).unwrap(), test_data);
    }
}
