//! Token types for fungible tokens (FT) and non-fungible tokens (NFT).

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// Token identifier (hex string derived from creator + name + symbol + timestamp).
pub type TokenId = String;

/// Information about a fungible token.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TokenInfo {
    pub token_id: TokenId,
    pub creator: String,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub total_supply: u64,
    /// Maximum supply (None = unlimited)
    pub max_supply: Option<u64>,
    pub is_mintable: bool,
    pub is_burnable: bool,
    pub created_at: u64,
}

/// Information about a non-fungible token.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NftInfo {
    pub nft_id: TokenId,
    /// The collection this NFT belongs to (references a TokenId from CreateToken)
    pub collection_id: String,
    pub owner: String,
    pub creator: String,
    pub name: String,
    /// URI pointing to metadata (IPFS, Arweave, or on-chain)
    pub metadata_uri: String,
    pub created_at: u64,
}

/// Registry of all tokens and their balances.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenRegistry {
    /// Fungible token info: token_id -> TokenInfo
    pub tokens: HashMap<TokenId, TokenInfo>,
    /// Fungible token balances: (address, token_id) -> balance
    pub balances: HashMap<(String, TokenId), u64>,
    /// NFT info: nft_id -> NftInfo
    pub nfts: HashMap<TokenId, NftInfo>,
}

impl TokenRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get a token balance for an address.
    pub fn get_token_balance(&self, address: &str, token_id: &str) -> u64 {
        self.balances
            .get(&(address.to_string(), token_id.to_string()))
            .copied()
            .unwrap_or(0)
    }

    /// Set a token balance for an address.
    pub fn set_token_balance(&mut self, address: &str, token_id: &str, balance: u64) {
        self.balances
            .insert((address.to_string(), token_id.to_string()), balance);
    }

    /// Get all token balances for an address.
    pub fn get_account_token_balances(&self, address: &str) -> HashMap<TokenId, u64> {
        self.balances
            .iter()
            .filter(|((addr, _), _)| addr == address)
            .map(|((_, token_id), &balance)| (token_id.clone(), balance))
            .collect()
    }

    /// Get all NFTs owned by an address.
    pub fn get_account_nfts(&self, address: &str) -> Vec<&NftInfo> {
        self.nfts
            .values()
            .filter(|nft| nft.owner == address)
            .collect()
    }
}

/// Derive a deterministic token ID from creator, name, symbol, and timestamp.
pub fn derive_token_id(creator: &str, name: &str, symbol: &str, timestamp: u64) -> TokenId {
    let mut hasher = Sha256::new();
    hasher.update(creator.as_bytes());
    hasher.update(name.as_bytes());
    hasher.update(symbol.as_bytes());
    hasher.update(timestamp.to_le_bytes());
    let result = hasher.finalize();
    hex::encode(&result[..20])
}

/// Derive a deterministic NFT ID from collection, creator, name, and timestamp.
pub fn derive_nft_id(collection_id: &str, creator: &str, name: &str, timestamp: u64) -> TokenId {
    let mut hasher = Sha256::new();
    hasher.update(collection_id.as_bytes());
    hasher.update(creator.as_bytes());
    hasher.update(name.as_bytes());
    hasher.update(timestamp.to_le_bytes());
    let result = hasher.finalize();
    hex::encode(&result[..20])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_token_id_deterministic() {
        let id1 = derive_token_id("alice", "MyToken", "MTK", 1000);
        let id2 = derive_token_id("alice", "MyToken", "MTK", 1000);
        assert_eq!(id1, id2);
        assert_eq!(id1.len(), 40);
    }

    #[test]
    fn test_derive_token_id_unique() {
        let id1 = derive_token_id("alice", "Token1", "T1", 1000);
        let id2 = derive_token_id("alice", "Token2", "T2", 1000);
        let id3 = derive_token_id("bob", "Token1", "T1", 1000);
        assert_ne!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_token_registry_balance() {
        let mut registry = TokenRegistry::new();
        assert_eq!(registry.get_token_balance("alice", "token1"), 0);

        registry.set_token_balance("alice", "token1", 100);
        assert_eq!(registry.get_token_balance("alice", "token1"), 100);
    }

    #[test]
    fn test_token_registry_account_balances() {
        let mut registry = TokenRegistry::new();
        registry.set_token_balance("alice", "t1", 100);
        registry.set_token_balance("alice", "t2", 200);
        registry.set_token_balance("bob", "t1", 50);

        let alice_balances = registry.get_account_token_balances("alice");
        assert_eq!(alice_balances.len(), 2);
        assert_eq!(alice_balances.get("t1"), Some(&100));
        assert_eq!(alice_balances.get("t2"), Some(&200));
    }

    #[test]
    fn test_derive_nft_id_deterministic() {
        let id1 = derive_nft_id("collection1", "alice", "NFT1", 1000);
        let id2 = derive_nft_id("collection1", "alice", "NFT1", 1000);
        assert_eq!(id1, id2);
    }
}
