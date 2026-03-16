// src/wallet/mod.rs
//! Wallet module for key management and encrypted storage
//!
//! Private keys are encrypted at rest using AES-256-GCM with a key derived
//! from a user password via Argon2id. The wallet file stores:
//! - The encrypted secret key (base64)
//! - A random salt for Argon2 (hex)
//! - A random nonce for AES-GCM (hex)

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use anyhow::{anyhow, Result};
use argon2::Argon2;
use ed25519_dalek::{SigningKey, VerifyingKey};
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;
use zeroize::Zeroizing;

/// Current wallet file format version (for forward compatibility)
const WALLET_FORMAT_VERSION: u32 = 2;

/// Wallet file structure (stored as JSON)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletFile {
    pub name: String,
    pub address: String,
    /// Format version (1 = plaintext hex, 2 = encrypted)
    #[serde(default = "default_version")]
    pub version: u32,
    /// Encrypted secret key bytes (base64). In v1, this is plain hex in `secret_key_hex`.
    #[serde(default)]
    pub encrypted_key: String,
    /// Argon2 salt (hex, 16 bytes)
    #[serde(default)]
    pub salt: String,
    /// AES-GCM nonce (hex, 12 bytes)
    #[serde(default)]
    pub nonce: String,
    /// Legacy field for v1 wallets (plain hex secret key). Kept for migration support.
    #[serde(default)]
    pub secret_key_hex: String,
    pub created_at: String,
}

fn default_version() -> u32 {
    1
}

/// In-memory wallet representation
pub struct Wallet {
    pub name: String,
    pub address: String,
    pub signing_key: SigningKey,
}

impl Wallet {
    /// Generate a new wallet with random keypair
    pub fn generate(name: String) -> Self {
        let signing_key = SigningKey::generate(&mut OsRng);
        let address = Self::derive_address(&signing_key.verifying_key());

        Self {
            name,
            address,
            signing_key,
        }
    }

    /// Derive address from public key (first 20 bytes of SHA256 hash, hex encoded)
    pub fn derive_address(pubkey: &VerifyingKey) -> String {
        let mut hasher = Sha256::new();
        hasher.update(pubkey.to_bytes());
        let result = hasher.finalize();
        // Take first 20 bytes -> 40 hex chars
        hex::encode(&result[0..20])
    }

    /// Get the verifying (public) key
    pub fn verifying_key(&self) -> VerifyingKey {
        self.signing_key.verifying_key()
    }

    /// Save wallet to file with password-based encryption.
    ///
    /// Uses Argon2id to derive a 256-bit key from the password,
    /// then AES-256-GCM to encrypt the 32-byte Ed25519 secret key.
    pub fn save_encrypted(&self, wallet_dir: &PathBuf, password: &str) -> Result<PathBuf> {
        fs::create_dir_all(wallet_dir)?;

        let secret_bytes = Zeroizing::new(self.signing_key.to_bytes());

        // Generate random salt (16 bytes) and nonce (12 bytes)
        let mut salt = [0u8; 16];
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut salt);
        OsRng.fill_bytes(&mut nonce_bytes);

        // Derive 256-bit encryption key from password via Argon2id
        let mut derived_key = Zeroizing::new([0u8; 32]);
        Argon2::default()
            .hash_password_into(password.as_bytes(), &salt, &mut derived_key[..])
            .map_err(|e| anyhow!("Argon2 key derivation failed: {}", e))?;

        // Encrypt the secret key with AES-256-GCM
        let cipher = Aes256Gcm::new_from_slice(&derived_key[..])
            .map_err(|e| anyhow!("AES cipher init failed: {}", e))?;
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = cipher
            .encrypt(nonce, &secret_bytes[..])
            .map_err(|e| anyhow!("Encryption failed: {}", e))?;

        let wallet_file = WalletFile {
            name: self.name.clone(),
            address: self.address.clone(),
            version: WALLET_FORMAT_VERSION,
            encrypted_key: base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                &ciphertext,
            ),
            salt: hex::encode(salt),
            nonce: hex::encode(nonce_bytes),
            secret_key_hex: String::new(), // not used in v2
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        let filename = format!("{}.json", self.address);
        let filepath = wallet_dir.join(&filename);

        let json = serde_json::to_string_pretty(&wallet_file)?;
        fs::write(&filepath, json)?;

        Ok(filepath)
    }

    /// Load wallet from an encrypted file (v2) or legacy plaintext file (v1).
    ///
    /// For v2 files, `password` is required.
    /// For v1 (legacy) files, the password is ignored and the plain hex key is loaded directly.
    pub fn load_encrypted(filepath: &PathBuf, password: &str) -> Result<Self> {
        let content = fs::read_to_string(filepath)?;
        let wallet_file: WalletFile = serde_json::from_str(&content)?;

        let signing_key = if wallet_file.version >= 2 {
            // Decrypt
            let salt_bytes =
                hex::decode(&wallet_file.salt).map_err(|e| anyhow!("Invalid salt hex: {}", e))?;
            let salt: [u8; 16] = salt_bytes
                .as_slice()
                .try_into()
                .map_err(|_| anyhow!("Invalid salt length"))?;

            let nonce_bytes =
                hex::decode(&wallet_file.nonce).map_err(|e| anyhow!("Invalid nonce hex: {}", e))?;
            let nonce_bytes: [u8; 12] = nonce_bytes
                .as_slice()
                .try_into()
                .map_err(|_| anyhow!("Invalid nonce length"))?;

            let ciphertext = base64::Engine::decode(
                &base64::engine::general_purpose::STANDARD,
                &wallet_file.encrypted_key,
            )
            .map_err(|e| anyhow!("Invalid encrypted_key base64: {}", e))?;

            // Re-derive key from password
            let mut derived_key = Zeroizing::new([0u8; 32]);
            Argon2::default()
                .hash_password_into(password.as_bytes(), &salt, &mut derived_key[..])
                .map_err(|e| anyhow!("Argon2 key derivation failed: {}", e))?;

            // Decrypt
            let cipher = Aes256Gcm::new_from_slice(&derived_key[..])
                .map_err(|e| anyhow!("AES cipher init failed: {}", e))?;
            let nonce = Nonce::from_slice(&nonce_bytes);
            let plaintext = cipher
                .decrypt(nonce, ciphertext.as_slice())
                .map_err(|_| anyhow!("Decryption failed: incorrect password or corrupted file"))?;
            let plaintext = Zeroizing::new(plaintext);

            let secret_array: [u8; 32] = plaintext
                .as_slice()
                .try_into()
                .map_err(|_| anyhow!("Invalid decrypted secret key length"))?;
            let secret_array = Zeroizing::new(secret_array);

            SigningKey::from_bytes(&secret_array)
        } else {
            // Legacy v1: plain hex secret key
            let secret_bytes = hex::decode(&wallet_file.secret_key_hex)?;
            let secret_bytes = Zeroizing::new(secret_bytes);
            let secret_array: [u8; 32] = secret_bytes
                .as_slice()
                .try_into()
                .map_err(|_| anyhow!("Invalid secret key length"))?;
            let secret_array = Zeroizing::new(secret_array);

            SigningKey::from_bytes(&secret_array)
        };

        Ok(Self {
            name: wallet_file.name,
            address: wallet_file.address,
            signing_key,
        })
    }

    /// Load wallet by address from wallet directory (encrypted)
    pub fn load_by_address(wallet_dir: &PathBuf, address: &str, password: &str) -> Result<Self> {
        let filename = format!("{}.json", address);
        let filepath = wallet_dir.join(&filename);
        Self::load_encrypted(&filepath, password)
    }
}

/// Get the default wallet directory
pub fn default_wallet_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".netchain")
        .join("wallets")
}

/// List all wallets in a directory (does not require password -- only reads metadata)
pub fn list_wallets(wallet_dir: &PathBuf) -> Result<Vec<WalletFile>> {
    let mut wallets = Vec::new();

    if !wallet_dir.exists() {
        return Ok(wallets);
    }

    for entry in fs::read_dir(wallet_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().map(|e| e == "json").unwrap_or(false) {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(wallet_file) = serde_json::from_str::<WalletFile>(&content) {
                    wallets.push(wallet_file);
                }
            }
        }
    }

    Ok(wallets)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_wallet_generation() {
        let wallet = Wallet::generate("test_wallet".to_string());

        assert_eq!(wallet.name, "test_wallet");
        assert_eq!(wallet.address.len(), 40); // 20 bytes = 40 hex chars
    }

    #[test]
    fn test_wallet_encrypted_save_and_load() {
        let dir = tempdir().unwrap();
        let wallet_dir = dir.path().to_path_buf();

        let wallet = Wallet::generate("test_wallet".to_string());
        let original_address = wallet.address.clone();
        let original_pubkey = wallet.verifying_key();

        // Save with password
        wallet
            .save_encrypted(&wallet_dir, "my_secure_password")
            .unwrap();

        // Load with correct password
        let loaded =
            Wallet::load_by_address(&wallet_dir, &original_address, "my_secure_password").unwrap();

        assert_eq!(loaded.name, "test_wallet");
        assert_eq!(loaded.address, original_address);
        assert_eq!(loaded.verifying_key(), original_pubkey);
    }

    #[test]
    fn test_wallet_wrong_password_fails() {
        let dir = tempdir().unwrap();
        let wallet_dir = dir.path().to_path_buf();

        let wallet = Wallet::generate("test_wallet".to_string());
        wallet
            .save_encrypted(&wallet_dir, "correct_password")
            .unwrap();

        // Load with wrong password should fail
        let result = Wallet::load_by_address(&wallet_dir, &wallet.address, "wrong_password");
        assert!(result.is_err());
    }

    #[test]
    fn test_wallet_rejects_invalid_nonce_length() {
        let dir = tempdir().unwrap();
        let wallet_dir = dir.path().to_path_buf();

        let wallet = Wallet::generate("test_wallet".to_string());
        let address = wallet.address.clone();

        // Save with password
        let path = wallet
            .save_encrypted(&wallet_dir, "my_secure_password")
            .unwrap();

        // Corrupt the wallet file nonce to an invalid (non-12-byte) value.
        let mut value: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        value["nonce"] = serde_json::json!("00"); // 1 byte nonce
        fs::write(&path, serde_json::to_string_pretty(&value).unwrap()).unwrap();

        let result = Wallet::load_by_address(&wallet_dir, &address, "my_secure_password");
        assert!(result.is_err());
    }

    #[test]
    fn test_address_derivation_consistency() {
        let wallet = Wallet::generate("test".to_string());
        let derived = Wallet::derive_address(&wallet.verifying_key());

        assert_eq!(wallet.address, derived);
    }
}
