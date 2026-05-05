//! Contract state persistence using sled.

use crate::contract::ContractInfo;
use sled::Db;
use std::collections::HashMap;

const CONTRACTS_TREE: &str = "contracts";
const CONTRACT_STATE_TREE: &str = "contract_state";

/// Manages contract persistence in sled.
pub struct ContractStorage {
    contracts: sled::Tree,
    state: sled::Tree,
}

impl ContractStorage {
    /// Open or create the contract storage trees.
    pub fn open(db: &Db) -> Result<Self, sled::Error> {
        let contracts = db.open_tree(CONTRACTS_TREE)?;
        let state = db.open_tree(CONTRACT_STATE_TREE)?;
        Ok(Self { contracts, state })
    }

    /// Save a contract's info.
    pub fn save_contract(&self, contract: &ContractInfo) -> Result<(), sled::Error> {
        let json = serde_json::to_vec(contract).expect("ContractInfo serialization failed");
        self.contracts.insert(contract.address.as_bytes(), json)?;
        Ok(())
    }

    /// Load a contract's info by address.
    pub fn load_contract(&self, address: &str) -> Result<Option<ContractInfo>, sled::Error> {
        match self.contracts.get(address.as_bytes())? {
            Some(bytes) => {
                let contract: ContractInfo =
                    serde_json::from_slice(&bytes).expect("ContractInfo deserialization failed");
                Ok(Some(contract))
            }
            None => Ok(None),
        }
    }

    /// List all deployed contracts.
    pub fn list_contracts(&self) -> Result<Vec<ContractInfo>, sled::Error> {
        let mut contracts = Vec::new();
        for entry in self.contracts.iter() {
            let (_, bytes) = entry?;
            let contract: ContractInfo =
                serde_json::from_slice(&bytes).expect("ContractInfo deserialization failed");
            contracts.push(contract);
        }
        Ok(contracts)
    }

    /// Read a value from contract storage.
    pub fn read_storage(
        &self,
        contract_address: &str,
        key: &[u8],
    ) -> Result<Option<Vec<u8>>, sled::Error> {
        let storage_key = make_storage_key(contract_address, key);
        match self.state.get(&storage_key)? {
            Some(bytes) => Ok(Some(bytes.to_vec())),
            None => Ok(None),
        }
    }

    /// Write a value to contract storage.
    pub fn write_storage(
        &self,
        contract_address: &str,
        key: &[u8],
        value: &[u8],
    ) -> Result<(), sled::Error> {
        let storage_key = make_storage_key(contract_address, key);
        self.state.insert(&storage_key, value)?;
        Ok(())
    }

    /// Delete a value from contract storage.
    pub fn delete_storage(&self, contract_address: &str, key: &[u8]) -> Result<(), sled::Error> {
        let storage_key = make_storage_key(contract_address, key);
        self.state.remove(&storage_key)?;
        Ok(())
    }

    /// Apply a batch of storage changes from VM execution.
    pub fn apply_storage_changes(
        &self,
        contract_address: &str,
        changes: &HashMap<Vec<u8>, Option<Vec<u8>>>,
    ) -> Result<(), sled::Error> {
        for (key, value) in changes {
            match value {
                Some(val) => self.write_storage(contract_address, key, val)?,
                None => self.delete_storage(contract_address, key)?,
            }
        }
        Ok(())
    }

    /// Load the full storage snapshot for a contract (for VM context).
    pub fn load_contract_storage(
        &self,
        contract_address: &str,
    ) -> Result<HashMap<Vec<u8>, Vec<u8>>, sled::Error> {
        let prefix = format!("{}:", contract_address);
        let mut storage = HashMap::new();
        for entry in self.state.scan_prefix(prefix.as_bytes()) {
            let (key, value) = entry?;
            // Strip the contract address prefix
            let storage_key = key[prefix.len()..].to_vec();
            storage.insert(storage_key, value.to_vec());
        }
        Ok(storage)
    }
}

/// Create a composite storage key: contract_address ++ ":" ++ key
fn make_storage_key(contract_address: &str, key: &[u8]) -> Vec<u8> {
    let mut storage_key = Vec::with_capacity(contract_address.len() + 1 + key.len());
    storage_key.extend_from_slice(contract_address.as_bytes());
    storage_key.push(b':');
    storage_key.extend_from_slice(key);
    storage_key
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_and_load_contract() {
        let db = sled::Config::new().temporary(true).open().unwrap();
        let storage = ContractStorage::open(&db).unwrap();

        let contract = ContractInfo {
            address: "contract1".to_string(),
            deployer: "alice".to_string(),
            code_hash: "abc123".to_string(),
            created_at: 100,
        };

        storage.save_contract(&contract).unwrap();
        let loaded = storage.load_contract("contract1").unwrap().unwrap();
        assert_eq!(loaded, contract);
    }

    #[test]
    fn test_contract_not_found() {
        let db = sled::Config::new().temporary(true).open().unwrap();
        let storage = ContractStorage::open(&db).unwrap();
        let result = storage.load_contract("nonexistent").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_storage_read_write() {
        let db = sled::Config::new().temporary(true).open().unwrap();
        let storage = ContractStorage::open(&db).unwrap();

        let key = b"counter";
        let value = b"\x01\x00\x00\x00";

        storage.write_storage("contract1", key, value).unwrap();
        let loaded = storage.read_storage("contract1", key).unwrap().unwrap();
        assert_eq!(loaded, value);
    }

    #[test]
    fn test_storage_delete() {
        let db = sled::Config::new().temporary(true).open().unwrap();
        let storage = ContractStorage::open(&db).unwrap();

        storage
            .write_storage("contract1", b"key", b"value")
            .unwrap();
        assert!(storage.read_storage("contract1", b"key").unwrap().is_some());

        storage.delete_storage("contract1", b"key").unwrap();
        assert!(storage.read_storage("contract1", b"key").unwrap().is_none());
    }

    #[test]
    fn test_apply_storage_changes() {
        let db = sled::Config::new().temporary(true).open().unwrap();
        let storage = ContractStorage::open(&db).unwrap();

        let mut changes = HashMap::new();
        changes.insert(b"key1".to_vec(), Some(b"value1".to_vec()));
        changes.insert(b"key2".to_vec(), Some(b"value2".to_vec()));

        storage
            .apply_storage_changes("contract1", &changes)
            .unwrap();

        assert_eq!(
            storage.read_storage("contract1", b"key1").unwrap(),
            Some(b"value1".to_vec())
        );
        assert_eq!(
            storage.read_storage("contract1", b"key2").unwrap(),
            Some(b"value2".to_vec())
        );
    }

    #[test]
    fn test_list_contracts() {
        let db = sled::Config::new().temporary(true).open().unwrap();
        let storage = ContractStorage::open(&db).unwrap();

        let c1 = ContractInfo {
            address: "c1".to_string(),
            deployer: "alice".to_string(),
            code_hash: "h1".to_string(),
            created_at: 1,
        };
        let c2 = ContractInfo {
            address: "c2".to_string(),
            deployer: "bob".to_string(),
            code_hash: "h2".to_string(),
            created_at: 2,
        };

        storage.save_contract(&c1).unwrap();
        storage.save_contract(&c2).unwrap();

        let contracts = storage.list_contracts().unwrap();
        assert_eq!(contracts.len(), 2);
    }
}
