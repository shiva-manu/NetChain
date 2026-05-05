//! Smart contract types and configuration.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Information about a deployed smart contract.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContractInfo {
    /// Contract address (derived from deployer + nonce)
    pub address: String,
    /// Address of the deployer
    pub deployer: String,
    /// SHA-256 hash of the WASM bytecode (hex)
    pub code_hash: String,
    /// Block height at which the contract was deployed
    pub created_at: u64,
}

/// Gas configuration for contract execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasConfig {
    /// Maximum gas allowed per transaction
    pub gas_limit: u64,
    /// Gas price in native tokens per gas unit
    pub gas_price: u64,
    /// Gas cost for reading from contract storage
    pub storage_read_cost: u64,
    /// Gas cost for writing to contract storage
    pub storage_write_cost: u64,
    /// Gas cost for a contract call
    pub call_cost: u64,
}

impl Default for GasConfig {
    fn default() -> Self {
        Self {
            gas_limit: 1_000_000,
            gas_price: 1,
            storage_read_cost: 100,
            storage_write_cost: 500,
            call_cost: 200,
        }
    }
}

/// Derive a contract address from deployer address and nonce.
/// Uses SHA-256(deployer ++ nonce_bytes)[0..20] hex-encoded.
pub fn derive_contract_address(deployer: &str, nonce: u64) -> String {
    let mut hasher = Sha256::new();
    hasher.update(deployer.as_bytes());
    hasher.update(nonce.to_le_bytes());
    let result = hasher.finalize();
    hex::encode(&result[..20])
}

/// Compute the code hash of WASM bytecode.
pub fn code_hash(wasm_code: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(wasm_code);
    let result = hasher.finalize();
    hex::encode(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_contract_address_deterministic() {
        let addr1 = derive_contract_address("alice", 0);
        let addr2 = derive_contract_address("alice", 0);
        assert_eq!(addr1, addr2);
        assert_eq!(addr1.len(), 40); // 20 bytes hex
    }

    #[test]
    fn test_derive_contract_address_unique() {
        let addr1 = derive_contract_address("alice", 0);
        let addr2 = derive_contract_address("alice", 1);
        let addr3 = derive_contract_address("bob", 0);
        assert_ne!(addr1, addr2);
        assert_ne!(addr1, addr3);
    }

    #[test]
    fn test_code_hash_deterministic() {
        let code = vec![0x00, 0x61, 0x73, 0x6d]; // WASM magic bytes
        let h1 = code_hash(&code);
        let h2 = code_hash(&code);
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64); // SHA-256 hex
    }
}
