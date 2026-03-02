use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::transaction::SignedTransaction;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub index: u64,
    pub timestamp: DateTime<Utc>,
    pub transactions: Vec<SignedTransaction>,
    pub previous_hash: String,
    pub hash: String,
}

impl Block {
    pub fn new(index: u64, transactions: Vec<SignedTransaction>, previous_hash: String) -> Self {
        let timestamp = Utc::now();
        let hash = Self::calculate_hash(index, &timestamp, &transactions, &previous_hash);
        Self {
            index,
            timestamp,
            transactions,
            previous_hash,
            hash,
        }
    }

    pub fn calculate_hash(
        index: u64,
        timestamp: &DateTime<Utc>,
        transactions: &[SignedTransaction],
        previous_hash: &str,
    ) -> String {
        // Deterministic hash over block header + full transaction list.
        let payload = serde_json::json!({
            "index": index,
            "timestamp": timestamp.to_rfc3339(),
            "transactions": transactions,
            "previous_hash": previous_hash,
        })
        .to_string();

        let mut hasher = Sha256::new();
        hasher.update(payload.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}
