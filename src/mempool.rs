use crate::state::{State, StateError};
use crate::transaction::SignedTransaction;
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug)]
pub enum MempoolError {
    InvalidTransaction(StateError),
    DuplicateTransaction,
    NonceTooLow,
}

#[derive(Debug)]
pub struct Mempool {
    txs: HashMap<String, SignedTransaction>,
    seen: HashSet<String>,
    by_sender: HashMap<String, VecDeque<String>>,
}

impl Mempool {
    pub fn new() -> Self {
        Self {
            txs: HashMap::new(),
            seen: HashSet::new(),
            by_sender: HashMap::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.txs.len()
    }

    pub fn add_transaction(
        &mut self,
        tx: SignedTransaction,
        state: &State,
    ) -> Result<(), MempoolError> {
        let tx_hash = tx.tx_hash_hex();

        if self.seen.contains(&tx_hash) {
            return Err(MempoolError::DuplicateTransaction);
        }

        state
            .validate_transaction(&tx)
            .map_err(MempoolError::InvalidTransaction)?;

        let sender = tx.tx.sender.clone();
        let nonce = tx.tx.nonce;

        if let Some(queue) = self.by_sender.get(&sender) {
            if let Some(last_hash) = queue.back() {
                let last_tx = self.txs.get(last_hash).expect("tx hash should exist");
                if nonce <= last_tx.tx.nonce {
                    return Err(MempoolError::NonceTooLow);
                }
            }
        }

        self.seen.insert(tx_hash.clone());
        self.txs.insert(tx_hash.clone(), tx);
        self.by_sender.entry(sender).or_default().push_back(tx_hash);

        Ok(())
    }

    pub fn remove_transaction(&mut self, tx_hash: &str) {
        if let Some(tx) = self.txs.remove(tx_hash) {
            self.seen.remove(tx_hash);
            if let Some(queue) = self.by_sender.get_mut(&tx.tx.sender) {
                queue.retain(|h| h != tx_hash);
                if queue.is_empty() {
                    self.by_sender.remove(&tx.tx.sender);
                }
            }
        }
    }

    pub fn remove_transactions(&mut self, txs: &[SignedTransaction]) {
        for tx in txs {
            self.remove_transaction(&tx.tx_hash_hex());
        }
    }

    pub fn select_for_block(&self, max_txs: usize) -> Vec<SignedTransaction> {
        let mut candidates: Vec<&SignedTransaction> = self.txs.values().collect();
        candidates.sort_by(|a, b| b.tx.fee.cmp(&a.tx.fee));
        candidates.into_iter().take(max_txs).cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::State;
    use crate::transaction::{generate_ed25519_keypair, pubkey_to_address_hex, Transaction};

    #[test]
    fn test_add_and_remove_tx() {
        let kp = generate_ed25519_keypair();
        let addr = pubkey_to_address_hex(&kp.public);

        let state = State::with_genesis(vec![(addr.clone(), 1000)]);
        let mut mempool = Mempool::new();

        let tx = Transaction::new(addr.clone(), "bob".into(), 100, 1, 0, None);
        let signed = crate::transaction::SignedTransaction::sign_with_keypair(&tx, &kp);

        assert!(mempool.add_transaction(signed.clone(), &state).is_ok());
        assert_eq!(mempool.len(), 1);

        mempool.remove_transaction(&signed.tx_hash_hex());
        assert_eq!(mempool.len(), 0);
    }

    #[test]
    fn test_duplicate_tx_rejected() {
        let kp = generate_ed25519_keypair();
        let addr = pubkey_to_address_hex(&kp.public);

        let state = State::with_genesis(vec![(addr.clone(), 1000)]);
        let mut mempool = Mempool::new();

        let tx = Transaction::new(addr.clone(), "bob".into(), 50, 1, 0, None);
        let signed = crate::transaction::SignedTransaction::sign_with_keypair(&tx, &kp);

        assert!(mempool.add_transaction(signed.clone(), &state).is_ok());
        assert!(matches!(
            mempool.add_transaction(signed, &state),
            Err(MempoolError::DuplicateTransaction)
        ));
    }
}
