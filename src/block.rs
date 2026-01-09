use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub index: u64,
    pub timestamp: DateTime<Utc>,
    pub data: String,
    pub previous_hash: String,
    pub hash: String,
}

impl Block {
    pub fn new(index: u64, data: String, previous_hash: String) -> Self {
        let timestamp = Utc::now();
        let hash = Self::calculate_hash(index, &timestamp, &data, &previous_hash);
        Self {
            index,
            timestamp,
            data,
            previous_hash,
            hash,
        }
    }

    pub fn calculate_hash(
        index: u64,
        timestamp: &DateTime<Utc>,
        data: &str,
        previous_hash: &str,
    ) -> String {
        // Simple hash over fields (Json encoding)
        let payload = serde_json::json!({
            "index": index,
            "timestamp": timestamp.to_rfc3339(),
            "data": data,
            "previous_hash": previous_hash,
        })
        .to_string();

        let mut hasher = Sha256::new();
        hasher.update(payload.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}
