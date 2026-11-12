// src/node/node_key.rs
//! Persistent node identity.
//!
//! Every node owns one Ed25519 key stored in `<data_dir>/node_key` (hex-encoded secret, 0600).
//! The same key is used for:
//! - the node's on-chain address (block rewards, staking, metric attestations),
//! - signing the blocks it proposes,
//! - its libp2p identity, so the PeerId is stable across restarts and any peer can
//!   derive the node's on-chain address from its PeerId.

use anyhow::{anyhow, Context, Result};
use ed25519_dalek::{SigningKey, VerifyingKey};
use libp2p::identity;
use libp2p::PeerId;
use std::fs;
use std::path::{Path, PathBuf};
use zeroize::Zeroize;

use crate::transaction::{generate_ed25519_keypair, pubkey_to_address_hex};

pub const NODE_KEY_FILE: &str = "node_key";

pub fn node_key_path(data_dir: &Path) -> PathBuf {
    data_dir.join(NODE_KEY_FILE)
}

/// Load the node key from `path`, or create a new one if the file does not exist.
pub fn load_or_create(path: &Path) -> Result<SigningKey> {
    if path.exists() {
        return load(path);
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let key = generate_ed25519_keypair();
    write_secret(path, &key)?;
    Ok(key)
}

/// Load an existing key file (hex-encoded 32-byte Ed25519 secret).
pub fn load(path: &Path) -> Result<SigningKey> {
    let mut contents = fs::read_to_string(path)
        .with_context(|| format!("failed to read key file {}", path.display()))?;
    let key = from_hex(&contents).with_context(|| format!("bad key file {}", path.display()));
    contents.zeroize();
    key
}

/// Parse a hex-encoded 32-byte Ed25519 secret.
pub fn from_hex(secret_hex: &str) -> Result<SigningKey> {
    let mut bytes = hex::decode(secret_hex.trim()).map_err(|_| anyhow!("key is not hex"))?;
    let secret: Result<[u8; 32]> = bytes
        .as_slice()
        .try_into()
        .map_err(|_| anyhow!("key must be 32 bytes (64 hex characters)"));
    bytes.zeroize();
    Ok(SigningKey::from_bytes(&secret?))
}

fn write_secret(path: &Path, key: &SigningKey) -> Result<()> {
    let mut encoded = hex::encode(key.to_bytes());
    let result = write_private_file(path, encoded.as_bytes());
    encoded.zeroize();
    result.with_context(|| format!("failed to write key file {}", path.display()))
}

#[cfg(unix)]
fn write_private_file(path: &Path, contents: &[u8]) -> std::io::Result<()> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)?;
    file.write_all(contents)?;
    file.write_all(b"\n")
}

#[cfg(not(unix))]
fn write_private_file(path: &Path, contents: &[u8]) -> std::io::Result<()> {
    fs::write(path, contents)
}

/// On-chain address for a signing key.
pub fn address_of(key: &SigningKey) -> String {
    pubkey_to_address_hex(&key.verifying_key())
}

/// libp2p identity derived from the node key, so PeerId == node identity.
pub fn libp2p_keypair(key: &SigningKey) -> Result<identity::Keypair> {
    let mut secret = key.to_bytes();
    let keypair = identity::Keypair::ed25519_from_bytes(&mut secret)
        .map_err(|e| anyhow!("invalid ed25519 key for libp2p: {e}"))?;
    secret.zeroize();
    Ok(keypair)
}

/// Derive a peer's on-chain address from its PeerId (works for Ed25519 peer identities,
/// which is what every NetChain node uses).
pub fn address_from_peer_id(peer_id: &PeerId) -> Option<String> {
    let multihash = peer_id.as_ref();
    // Ed25519 PeerIds use the identity multihash: the digest is the protobuf-encoded key.
    if multihash.code() != 0 {
        return None;
    }
    let public = identity::PublicKey::try_decode_protobuf(multihash.digest()).ok()?;
    let ed = public.try_into_ed25519().ok()?;
    let verifying = VerifyingKey::from_bytes(&ed.to_bytes()).ok()?;
    Some(pubkey_to_address_hex(&verifying))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_round_trips_through_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = node_key_path(dir.path());
        let created = load_or_create(&path).unwrap();
        let loaded = load_or_create(&path).unwrap();
        assert_eq!(created.to_bytes(), loaded.to_bytes());
    }

    #[test]
    fn hex_keys_parse_and_reject_garbage() {
        let key = generate_ed25519_keypair();
        let parsed = from_hex(&hex::encode(key.to_bytes())).unwrap();
        assert_eq!(parsed.to_bytes(), key.to_bytes());
        assert!(from_hex("zz").is_err());
        assert!(from_hex("abcd").is_err());
    }

    #[test]
    fn peer_id_maps_back_to_address() {
        let key = generate_ed25519_keypair();
        let keypair = libp2p_keypair(&key).unwrap();
        let peer_id = PeerId::from(keypair.public());
        assert_eq!(address_from_peer_id(&peer_id), Some(address_of(&key)));
    }
}
