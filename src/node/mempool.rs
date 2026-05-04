use crate::state::{State, StateError};
use crate::transaction::SignedTransaction;
use std::collections::{HashMap, HashSet, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH};

/// Errors returned by the mempool
#[derive(Debug)]
pub enum MemPoolError {
    InvalidTransaction(StateError),
    DuplicateTransaction,
    NonceTooLow,
}

/// In-memory transaction pool
#[derive(Debug)]
pub struct Mempool {
    /// tx_hash -> SignedTransaction
    txs: HashMap<String, SignedTransaction>,

    /// Track tx hashes to prevent duplicates quickly
    seen: HashSet<String>,

    /// sender -> ordered queue of tx hashes (nonce order)
    by_sender: HashMap<String, VecDeque<String>>,

    /// tx_hash -> unix timestamp when the tx was inserted into the mempool
    inserted_at: HashMap<String, u64>,
}

/// Maximum serialized block size in bytes (1 MB default).
/// This prevents oversized blocks from being produced or accepted.
pub const MAX_BLOCK_SIZE_BYTES: usize = 1024 * 1024;

impl Mempool {
    /// Create empty mempool
    pub fn new() -> Self {
        Self {
            txs: HashMap::new(),
            seen: HashSet::new(),
            by_sender: HashMap::new(),
            inserted_at: HashMap::new(),
        }
    }

    /// Number of transactions in pool
    pub fn len(&self) -> usize {
        self.txs.len()
    }

    /// Add a transaction to the mempool after validation
    pub fn add_transaction(
        &mut self,
        tx: SignedTransaction,
        state: &State,
    ) -> Result<(), MemPoolError> {
        let tx_hash = tx.tx_hash_hex();

        // Prevent duplicates
        if self.seen.contains(&tx_hash) {
            return Err(MemPoolError::DuplicateTransaction);
        }

        // Validate against current state
        state
            .validate_transaction(&tx)
            .map_err(MemPoolError::InvalidTransaction)?;

        let sender = tx.tx.sender.clone();
        let nonce = tx.tx.nonce;

        // Enforce monotonic nonce ordering per sender
        if let Some(queue) = self.by_sender.get(&sender) {
            if let Some(last_hash) = queue.back() {
                let last_tx = self.txs.get(last_hash).expect("tx must exist");
                if nonce <= last_tx.tx.nonce {
                    return Err(MemPoolError::NonceTooLow);
                }
            }
        }

        // Insert
        let now = current_unix_timestamp();
        self.seen.insert(tx_hash.clone());
        self.inserted_at.insert(tx_hash.clone(), now);
        self.txs.insert(tx_hash.clone(), tx);
        self.by_sender
            .entry(sender)
            .or_insert_with(VecDeque::new)
            .push_back(tx_hash);

        Ok(())
    }

    /// Remove a transaction (called after block inclusion)
    pub fn remove_transaction(&mut self, tx_hash: &str) {
        if let Some(tx) = self.txs.remove(tx_hash) {
            self.seen.remove(tx_hash);
            self.inserted_at.remove(tx_hash);
            if let Some(queue) = self.by_sender.get_mut(&tx.tx.sender) {
                queue.retain(|h| h != tx_hash);
                if queue.is_empty() {
                    self.by_sender.remove(&tx.tx.sender);
                }
            }
        }
    }

    /// Remove all txs included in a block
    pub fn remove_transactions(&mut self, txs: &[SignedTransaction]) {
        for tx in txs {
            self.remove_transaction(&tx.tx_hash_hex());
        }
    }

    /// Remove transactions that have been in the mempool longer than `ttl_secs`.
    ///
    /// Returns the number of expired transactions removed.
    pub fn expire_old(&mut self, now: u64, ttl_secs: u64) -> usize {
        let expired_hashes: Vec<String> = self
            .inserted_at
            .iter()
            .filter(|(_hash, &inserted)| now.saturating_sub(inserted) >= ttl_secs)
            .map(|(hash, _)| hash.clone())
            .collect();

        let count = expired_hashes.len();
        for hash in expired_hashes {
            self.remove_transaction(&hash);
        }
        count
    }

    /// Select transactions for block production
    /// - Highest-fee first (deterministic tie-breakers)
    /// - Enforces per-sender nonce ordering
    /// - Only selects transactions that are valid against the provided `state`
    /// - Enforces a maximum serialized block size to prevent DoS
    pub fn select_for_block(
        &self,
        max_txs: usize,
        state: &State,
        now: u64,
        max_block_size_bytes: usize,
    ) -> Vec<SignedTransaction> {
        if max_txs == 0 || self.txs.is_empty() {
            return Vec::new();
        }

        // Pre-compute serialized sizes for all transactions (cheap one-time cost).
        let config = bincode::config::standard()
            .with_fixed_int_encoding()
            .with_little_endian();
        let tx_sizes: HashMap<String, usize> = self
            .txs
            .iter()
            .map(|(hash, tx)| {
                let size = bincode::serde::encode_to_vec(tx, config)
                    .map(|v| v.len())
                    .unwrap_or(usize::MAX);
                (hash.clone(), size)
            })
            .collect();

        // Work on a clone so we can simulate sequential execution without mutating the real state.
        let mut working_state = state.clone();

        // Clone queues so we can pop stale heads without mutating the live mempool.
        let mut sender_queues = self.by_sender.clone();
        let mut senders: Vec<String> = sender_queues.keys().cloned().collect();
        // Deterministic iteration across nodes.
        senders.sort();

        let mut selected = Vec::new();
        let mut current_size: usize = 0;

        while selected.len() < max_txs {
            // Best candidate among currently-valid sender heads: (sender, tx_hash, fee)
            let mut best: Option<(String, String, u64)> = None;

            for sender in &senders {
                let Some(queue) = sender_queues.get_mut(sender) else {
                    continue;
                };

                // Drop stale transactions with nonce < expected, so a restarted node doesn't get stuck.
                loop {
                    let Some(front_hash) = queue.front() else {
                        break;
                    };
                    let Some(front_tx) = self.txs.get(front_hash) else {
                        // Should not happen, but keep going.
                        queue.pop_front();
                        continue;
                    };

                    let expected_nonce = working_state.get_nonce(sender);

                    if front_tx.tx.nonce < expected_nonce {
                        queue.pop_front();
                        continue;
                    }

                    // Nonce gap: can't include any txs from this sender right now.
                    if front_tx.tx.nonce > expected_nonce {
                        break;
                    }

                    // Nonce matches. Validate against working state (includes balance checks).
                    if working_state.validate_transaction_at(front_tx, now).is_ok() {
                        let fee = front_tx.tx.fee;
                        let candidate_sender = sender.clone();
                        let candidate_hash = front_hash.clone();

                        let better = match &best {
                            None => true,
                            Some((best_sender, best_hash, best_fee)) => {
                                fee > *best_fee
                                    || (fee == *best_fee
                                        && (candidate_sender < *best_sender
                                            || (candidate_sender == *best_sender
                                                && candidate_hash < *best_hash)))
                            }
                        };

                        if better {
                            best = Some((candidate_sender, candidate_hash, fee));
                        }
                    }

                    break;
                }
            }

            let Some((best_sender, best_hash, _best_fee)) = best else {
                break;
            };

            let Some(tx) = self.txs.get(&best_hash) else {
                // Should not happen; drop from the cloned queue to avoid looping forever.
                if let Some(queue) = sender_queues.get_mut(&best_sender) {
                    queue.pop_front();
                }
                continue;
            };

            // Apply to working state to advance nonces/balances for subsequent picks.
            if working_state.apply_transaction_at(tx, now).is_err() {
                // If this fails, treat it as not selectable and move on.
                if let Some(queue) = sender_queues.get_mut(&best_sender) {
                    queue.pop_front();
                }
                continue;
            }

            // Check if adding this tx would exceed the block size limit.
            let tx_size = tx_sizes.get(&best_hash).copied().unwrap_or(usize::MAX);
            if current_size + tx_size > max_block_size_bytes {
                // Block size limit reached. Stop selecting more transactions.
                break;
            }

            selected.push(tx.clone());
            current_size += tx_size;

            // Consume the head we just used.
            if let Some(queue) = sender_queues.get_mut(&best_sender) {
                if queue.front().map(|h| h == &best_hash).unwrap_or(false) {
                    queue.pop_front();
                } else {
                    // Defensive: if queues got out of sync, drop the tx by hash.
                    queue.retain(|h| h != &best_hash);
                }
            }
        }

        selected
    }
}

fn current_unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::State;
    use crate::transaction::{
        generate_ed25519_keypair, pubkey_to_address_hex, SignedTransaction, Transaction,
    };

    #[test]
    fn test_add_and_remove_tx() {
        let kp = generate_ed25519_keypair();
        let addr = pubkey_to_address_hex(&kp.verifying_key());

        let state = State::with_genesis(vec![(addr.clone(), 1000)]);
        let mut mempool = Mempool::new();

        let tx = Transaction::new(addr.clone(), "bob".into(), 100, 1, 0, None);
        let signed = SignedTransaction::sign_with_keypair(&tx, &kp);

        assert!(mempool.add_transaction(signed.clone(), &state).is_ok());
        assert_eq!(mempool.len(), 1);

        mempool.remove_transaction(&signed.tx_hash_hex());
        assert_eq!(mempool.len(), 0);
    }

    #[test]
    fn test_duplicate_tx_rejected() {
        let kp = generate_ed25519_keypair();
        let addr = pubkey_to_address_hex(&kp.verifying_key());

        let state = State::with_genesis(vec![(addr.clone(), 1000)]);
        let mut mempool = Mempool::new();

        let tx = Transaction::new(addr.clone(), "bob".into(), 50, 1, 0, None);
        let signed = SignedTransaction::sign_with_keypair(&tx, &kp);

        assert!(mempool.add_transaction(signed.clone(), &state).is_ok());
        assert!(matches!(
            mempool.add_transaction(signed, &state),
            Err(MemPoolError::DuplicateTransaction)
        ));
    }

    #[test]
    fn test_expire_old_removes_stale_txs() {
        let kp1 = generate_ed25519_keypair();
        let addr1 = pubkey_to_address_hex(&kp1.verifying_key());
        let kp2 = generate_ed25519_keypair();
        let addr2 = pubkey_to_address_hex(&kp2.verifying_key());

        let state = State::with_genesis(vec![(addr1.clone(), 10_000), (addr2.clone(), 10_000)]);
        let mut mempool = Mempool::new();

        let tx0 = Transaction::new(addr1.clone(), "bob".into(), 50, 1, 0, None);
        let signed0 = SignedTransaction::sign_with_keypair(&tx0, &kp1);
        assert!(mempool.add_transaction(signed0.clone(), &state).is_ok());

        let tx1 = Transaction::new(addr2.clone(), "bob".into(), 60, 1, 0, None);
        let signed1 = SignedTransaction::sign_with_keypair(&tx1, &kp2);
        assert!(mempool.add_transaction(signed1.clone(), &state).is_ok());
        assert_eq!(mempool.len(), 2);

        // With a TTL of 900s and "now" only 100s in the future, nothing should expire.
        let now = current_unix_timestamp();
        let expired = mempool.expire_old(now + 100, 900);
        assert_eq!(expired, 0);
        assert_eq!(mempool.len(), 2);

        // Fast-forward past TTL: everything should expire.
        let expired = mempool.expire_old(now + 901, 900);
        assert_eq!(expired, 2);
        assert_eq!(mempool.len(), 0);

        // The seen set should also be cleared so the same tx can be re-added.
        assert!(!mempool.seen.contains(&signed0.tx_hash_hex()));
        assert!(!mempool.seen.contains(&signed1.tx_hash_hex()));
    }

    #[test]
    fn test_expire_old_partial_expiry() {
        let kp1 = generate_ed25519_keypair();
        let addr1 = pubkey_to_address_hex(&kp1.verifying_key());
        let kp2 = generate_ed25519_keypair();
        let addr2 = pubkey_to_address_hex(&kp2.verifying_key());

        let state = State::with_genesis(vec![(addr1.clone(), 10_000), (addr2.clone(), 10_000)]);
        let mut mempool = Mempool::new();

        // Insert first tx
        let tx0 = Transaction::new(addr1.clone(), "bob".into(), 50, 1, 0, None);
        let signed0 = SignedTransaction::sign_with_keypair(&tx0, &kp1);
        assert!(mempool.add_transaction(signed0.clone(), &state).is_ok());

        // Manually backdate the first tx's insertion time by 1000s
        let hash0 = signed0.tx_hash_hex();
        let original_time = *mempool.inserted_at.get(&hash0).unwrap();
        mempool
            .inserted_at
            .insert(hash0.clone(), original_time.saturating_sub(1000));

        // Insert second tx from a different sender (at current time)
        let tx1 = Transaction::new(addr2.clone(), "bob".into(), 60, 1, 0, None);
        let signed1 = SignedTransaction::sign_with_keypair(&tx1, &kp2);
        assert!(mempool.add_transaction(signed1.clone(), &state).is_ok());
        assert_eq!(mempool.len(), 2);

        // Expire with TTL of 900s at current time. Only the backdated tx should expire.
        let now = current_unix_timestamp();
        let expired = mempool.expire_old(now, 900);
        assert_eq!(expired, 1);
        assert_eq!(mempool.len(), 1);
        assert!(mempool.txs.contains_key(&signed1.tx_hash_hex()));
        assert!(!mempool.txs.contains_key(&hash0));
    }
}
