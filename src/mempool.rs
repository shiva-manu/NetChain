use crate::state::{State, StateError};
use crate::transaction::SignedTransaction;
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug)]
pub enum MempoolError {
    InvalidTransaction(StateError),
    DuplicateTransaction,
    NonceGap,
    InsufficientBalance,
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

        tx.verify()
            .map_err(|_| MempoolError::InvalidTransaction(StateError::InvalidSignature))?;

        if tx.tx.amount == 0 {
            return Err(MempoolError::InvalidTransaction(StateError::ZeroAmount));
        }

        if !state.has_account(&tx.tx.sender) {
            return Err(MempoolError::InvalidTransaction(StateError::SenderNotFound));
        }

        let queue = self.by_sender.get(&tx.tx.sender);
        let expected_nonce = state.get_nonce(&tx.tx.sender) + queue.map_or(0, |q| q.len() as u64);
        if tx.tx.nonce != expected_nonce {
            return Err(MempoolError::NonceGap);
        }

        let reserved = queue
            .map(|q| {
                q.iter()
                    .filter_map(|h| self.txs.get(h))
                    .map(|t| t.tx.amount + t.tx.fee)
                    .sum::<u64>()
            })
            .unwrap_or(0);

        let sender_balance = state.get_balance(&tx.tx.sender);
        if reserved + tx.tx.amount + tx.tx.fee > sender_balance {
            return Err(MempoolError::InsufficientBalance);
        }

        self.seen.insert(tx_hash.clone());
        self.txs.insert(tx_hash.clone(), tx.clone());
        self.by_sender
            .entry(tx.tx.sender.clone())
            .or_default()
            .push_back(tx_hash);

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

    pub fn select_for_block(&self, state: &State, max_txs: usize) -> Vec<SignedTransaction> {
        let mut selected = Vec::new();
        let mut queue_positions: HashMap<String, usize> = HashMap::new();
        let mut next_nonce: HashMap<String, u64> = HashMap::new();

        for sender in self.by_sender.keys() {
            queue_positions.insert(sender.clone(), 0);
            next_nonce.insert(sender.clone(), state.get_nonce(sender));
        }

        while selected.len() < max_txs {
            let mut best: Option<(&String, &SignedTransaction)> = None;

            for (sender, queue) in &self.by_sender {
                let pos = *queue_positions.get(sender).unwrap_or(&0);
                if pos >= queue.len() {
                    continue;
                }

                let tx_hash = &queue[pos];
                let Some(tx) = self.txs.get(tx_hash) else {
                    continue;
                };

                let expected = *next_nonce.get(sender).unwrap_or(&state.get_nonce(sender));
                if tx.tx.nonce != expected {
                    continue;
                }

                match best {
                    None => best = Some((sender, tx)),
                    Some((_, current_best)) if tx.tx.fee > current_best.tx.fee => {
                        best = Some((sender, tx))
                    }
                    _ => {}
                }
            }

            let Some((sender, tx)) = best else {
                break;
            };

            selected.push(tx.clone());
            *queue_positions.entry(sender.clone()).or_insert(0) += 1;
            *next_nonce.entry(sender.clone()).or_insert(0) += 1;
        }

        selected
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
        let signed = SignedTransaction::sign_with_keypair(&tx, &kp);

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
        let signed = SignedTransaction::sign_with_keypair(&tx, &kp);

        assert!(mempool.add_transaction(signed.clone(), &state).is_ok());
        assert!(matches!(
            mempool.add_transaction(signed, &state),
            Err(MempoolError::DuplicateTransaction)
        ));
    }

    #[test]
    fn test_nonce_gap_rejected() {
        let kp = generate_ed25519_keypair();
        let addr = pubkey_to_address_hex(&kp.public);
        let state = State::with_genesis(vec![(addr.clone(), 1000)]);
        let mut mempool = Mempool::new();

        let tx = Transaction::new(addr, "bob".into(), 10, 1, 2, None);
        let signed = SignedTransaction::sign_with_keypair(&tx, &kp);
        assert!(matches!(
            mempool.add_transaction(signed, &state),
            Err(MempoolError::NonceGap)
        ));
    }

    #[test]
    fn test_select_respects_sender_nonce_order() {
        let kp1 = generate_ed25519_keypair();
        let a1 = pubkey_to_address_hex(&kp1.public);
        let kp2 = generate_ed25519_keypair();
        let a2 = pubkey_to_address_hex(&kp2.public);

        let state = State::with_genesis(vec![(a1.clone(), 1000), (a2.clone(), 1000)]);
        let mut mempool = Mempool::new();

        let t1 = SignedTransaction::sign_with_keypair(
            &Transaction::new(a1.clone(), "bob".into(), 10, 1, 0, None),
            &kp1,
        );
        let t2 = SignedTransaction::sign_with_keypair(
            &Transaction::new(a1.clone(), "bob".into(), 10, 50, 1, None),
            &kp1,
        );
        let t3 = SignedTransaction::sign_with_keypair(
            &Transaction::new(a2.clone(), "carol".into(), 10, 10, 0, None),
            &kp2,
        );

        mempool.add_transaction(t1.clone(), &state).unwrap();
        mempool.add_transaction(t2.clone(), &state).unwrap();
        mempool.add_transaction(t3.clone(), &state).unwrap();

        let picked = mempool.select_for_block(&state, 3);
        assert_eq!(picked.len(), 3);
        assert_eq!(picked[0].tx.sender, a2);
        assert_eq!(picked[1].tx.nonce, 0);
        assert_eq!(picked[2].tx.nonce, 1);
    }
}
