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

        if self.seen.contains(&tx_hash) {
            return Err(MempoolError::DuplicateTransaction);
        }

        tx.verify()
            .map_err(|_| MempoolError::InvalidTransaction(StateError::InvalidSignature))?;

        if tx.tx.amount == 0 {
            return Err(MempoolError::InvalidTransaction(StateError::ZeroAmount));
        }

        if state.get_balance(&tx.tx.sender) < tx.tx.amount + tx.tx.fee {
            return Err(MempoolError::InvalidTransaction(
                StateError::InsufficientBalance,
            ));
        }

        let sender = tx.tx.sender.clone();
        let nonce = tx.tx.nonce;
        let state_nonce = state.get_nonce(&sender);

        if let Some(queue) = self.by_sender.get(&sender) {
            if let Some(last_hash) = queue.back() {
                let last_tx = self.txs.get(last_hash).expect("tx hash should exist");
                if nonce != last_tx.tx.nonce + 1 {
                    return Err(MempoolError::NonceTooLow);
                }
            }
        } else if nonce != state_nonce {
            return Err(MempoolError::NonceTooLow);
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
        let mut next_nonce: HashMap<String, u64> = HashMap::new();
        let mut next_index: HashMap<String, usize> = HashMap::new();
        let mut ready: HashMap<String, SignedTransaction> = HashMap::new();

        for sender in self.by_sender.keys() {
            next_nonce.insert(sender.clone(), state.get_nonce(sender));
            next_index.insert(sender.clone(), 0);
            self.refresh_ready_for_sender(sender, &mut next_nonce, &mut next_index, &mut ready);
        }

        while selected.len() < max_txs && !ready.is_empty() {
            let best_sender = ready
                .iter()
                .max_by_key(|(_, tx)| tx.tx.fee)
                .map(|(sender, _)| sender.clone())
                .expect("ready map is non-empty");

            let chosen = ready
                .remove(&best_sender)
                .expect("selected sender should be in ready map");
            selected.push(chosen.clone());

            *next_nonce
                .get_mut(&best_sender)
                .expect("sender should have a nonce entry") += 1;
            self.refresh_ready_for_sender(
                &best_sender,
                &mut next_nonce,
                &mut next_index,
                &mut ready,
            );
        }

        selected
    }

    fn refresh_ready_for_sender(
        &self,
        sender: &str,
        next_nonce: &mut HashMap<String, u64>,
        next_index: &mut HashMap<String, usize>,
        ready: &mut HashMap<String, SignedTransaction>,
    ) {
        let Some(queue) = self.by_sender.get(sender) else {
            ready.remove(sender);
            return;
        };

        let idx = *next_index.get(sender).unwrap_or(&0);
        let Some(hash) = queue.get(idx) else {
            ready.remove(sender);
            return;
        };
        let tx = self.txs.get(hash).expect("tx hash should exist");

        if tx.tx.nonce == *next_nonce.get(sender).unwrap_or(&0) {
            ready.insert(sender.to_string(), tx.clone());
            next_index.insert(sender.to_string(), idx + 1);
        } else {
            ready.remove(sender);
        }
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
        let signed = SignedTransaction::sign_with_keypair(&tx, &kp);
        let signed = crate::transaction::SignedTransaction::sign_with_keypair(&tx, &kp);

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
        let gap_tx = Transaction::new(addr.clone(), "bob".into(), 50, 1, 1, None);
        let signed = crate::transaction::SignedTransaction::sign_with_keypair(&gap_tx, &kp);

        assert!(matches!(
            mempool.add_transaction(signed, &state),
            Err(MempoolError::NonceTooLow)
        ));
    }

    #[test]
    fn test_out_of_order_sender_tx_rejected() {
        let kp = generate_ed25519_keypair();
        let addr = pubkey_to_address_hex(&kp.public);
        let state = State::with_genesis(vec![(addr.clone(), 1000)]);
        let mut mempool = Mempool::new();

        let tx0 = Transaction::new(addr.clone(), "bob".into(), 10, 1, 0, None);
        let tx2 = Transaction::new(addr.clone(), "carol".into(), 10, 1, 2, None);
        let signed0 = crate::transaction::SignedTransaction::sign_with_keypair(&tx0, &kp);
        let signed2 = crate::transaction::SignedTransaction::sign_with_keypair(&tx2, &kp);

        assert!(mempool.add_transaction(signed0, &state).is_ok());
        assert!(matches!(
            mempool.add_transaction(signed2, &state),
            Err(MempoolError::NonceTooLow)
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
    fn test_valid_contiguous_sequence_accepted() {
        let kp = generate_ed25519_keypair();
        let addr = pubkey_to_address_hex(&kp.public);
        let state = State::with_genesis(vec![(addr.clone(), 1000)]);
        let mut mempool = Mempool::new();

        for nonce in 0..3 {
            let tx = Transaction::new(addr.clone(), format!("recv-{nonce}"), 10, 1, nonce, None);
            let signed = crate::transaction::SignedTransaction::sign_with_keypair(&tx, &kp);
            assert!(mempool.add_transaction(signed, &state).is_ok());
        }

        assert_eq!(mempool.len(), 3);
    }

    #[test]
    fn test_select_for_block_respects_executable_nonce_order() {
        let kp_a = generate_ed25519_keypair();
        let addr_a = pubkey_to_address_hex(&kp_a.public);
        let kp_b = generate_ed25519_keypair();
        let addr_b = pubkey_to_address_hex(&kp_b.public);

        let state = State::with_genesis(vec![(addr_a.clone(), 1000), (addr_b.clone(), 1000)]);
        let mut mempool = Mempool::new();

        let a0 = Transaction::new(addr_a.clone(), "x".into(), 10, 1, 0, None);
        let a1 = Transaction::new(addr_a.clone(), "y".into(), 10, 50, 1, None);
        let b0 = Transaction::new(addr_b.clone(), "z".into(), 10, 10, 0, None);

        assert!(mempool
            .add_transaction(
                crate::transaction::SignedTransaction::sign_with_keypair(&a0, &kp_a),
                &state
            )
            .is_ok());
        assert!(mempool
            .add_transaction(
                crate::transaction::SignedTransaction::sign_with_keypair(&a1, &kp_a),
                &state
            )
            .is_ok());
        assert!(mempool
            .add_transaction(
                crate::transaction::SignedTransaction::sign_with_keypair(&b0, &kp_b),
                &state
            )
            .is_ok());

        let selected = mempool.select_for_block(&state, 3);
        let a_nonces: Vec<u64> = selected
            .iter()
            .filter(|tx| tx.tx.sender == addr_a)
            .map(|tx| tx.tx.nonce)
            .collect();

        assert_eq!(a_nonces, vec![0, 1]);
    }
}
