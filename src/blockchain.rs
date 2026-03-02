use crate::block::Block;
use crate::state::{State, StateError};
use crate::transaction::SignedTransaction;
use crate::state::State;

#[derive(Debug, Clone)]
pub struct Blockchain {
    pub chain: Vec<Block>,
    pub state: State,
    genesis_allocations: Vec<(String, u64)>,
}

impl Blockchain {
    pub fn new() -> Self {
        Self::with_genesis(vec![])
    }

    pub fn with_genesis(genesis: Vec<(String, u64)>) -> Self {
        let mut bc = Blockchain {
            chain: Vec::new(),
            state: State::with_genesis(genesis.clone()),
            genesis_allocations: genesis,
        Self::with_genesis_state(vec![])
    }

    pub fn with_genesis_state(genesis_balances: Vec<(String, u64)>) -> Self {
        let mut bc = Blockchain {
            chain: Vec::new(),
            state: State::with_genesis(genesis_balances),
        };
        bc.chain.push(Self::genesis_block());
        bc
    }

    pub fn from_chain(chain: Vec<Block>) -> Result<Self, String> {
        if chain.is_empty() {
            return Err("Loaded chain is empty".into());
        }

        let bc = Self { chain };
        if bc.chain[0].index != 0 || bc.chain[0].previous_hash != "0" {
            return Err("Invalid genesis block".into());
        }

        if !bc.is_valid() {
            return Err("Loaded chain failed validation".into());
        }

        Ok(bc)
    }

    fn genesis_block() -> Block {
        Block::new(0, vec![], "0".to_string())
    }

    pub fn last_block(&self) -> &Block {
        self.chain
            .last()
            .expect("Blockchain must have at least one block")
    }

    pub fn add_block(&mut self, transactions: Vec<SignedTransaction>) -> Result<Block, String> {
        let last = self.last_block();
        let new_block = Block::new(last.index + 1, transactions, last.hash.clone());
        self.validate_and_add_block(new_block.clone())?;
        Ok(new_block)
    /// Used by local miner / validator
    pub fn add_block(&mut self, transactions: Vec<crate::transaction::SignedTransaction>) -> Block {
        let last = self.last_block();
        let new_block = Block::new(last.index + 1, transactions, last.hash.clone());
        self.chain.push(new_block.clone());
        new_block
    }

    pub fn validate_and_add_block(&mut self, block: Block) -> Result<(), String> {
        let last = self.last_block();

        if block.index != last.index + 1 {
            return Err("Invalid index".into());
        }

        if block.previous_hash != last.hash {
            return Err("Invalid previous hash".into());
        }

        let recalculated = Block::calculate_hash(
            block.index,
            &block.timestamp,
            &block.transactions,
            &block.previous_hash,
        );

        if block.hash != recalculated {
            return Err("Invalid block hash".into());
        }

        let mut state_candidate = self.state.clone();
        state_candidate
            .apply_transactions(&block.transactions)
            .map_err(|e| format!("Invalid block transactions: {}", state_error_message(&e)))?;

        self.state = state_candidate;
        let mut next_state = self.state.clone();
        for tx in &block.transactions {
            next_state
                .validate_transaction(tx)
                .map_err(|e| format!("Invalid transaction in block: {e:?}"))?;
            next_state
                .apply_transaction(tx)
                .map_err(|e| format!("Failed to apply transaction in block: {e:?}"))?;
        }

        self.chain.push(block);
        self.state = next_state;
        Ok(())
    }

    pub fn is_valid(&self) -> bool {
        let mut state = State::with_genesis(self.genesis_allocations.clone());

        for i in 0..self.chain.len() {
            let current = &self.chain[i];

            if i == 0 {
                continue;
            }

            let previous = &self.chain[i - 1];

            if current.previous_hash != previous.hash {
                return false;
            }

            let recalculated = Block::calculate_hash(
                current.index,
                &current.timestamp,
                &current.transactions,
                &current.previous_hash,
            );

            if current.hash != recalculated {
                return false;
            }

            if state.apply_transactions(&current.transactions).is_err() {
                return false;
            }
        }
        true
    }
}

fn state_error_message(err: &StateError) -> &'static str {
    match err {
        StateError::InsufficientBalance => "insufficient balance",
        StateError::InvalidNonce => "invalid nonce",
        StateError::InvalidSignature => "invalid signature",
        StateError::ZeroAmount => "zero amount",
        StateError::SenderNotFound => "sender not found",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transaction::{generate_ed25519_keypair, pubkey_to_address_hex, Transaction};

    #[test]
    fn accepts_valid_transaction_block() {
        let kp = generate_ed25519_keypair();
        let sender = pubkey_to_address_hex(&kp.public);
        let mut bc = Blockchain::with_genesis(vec![(sender.clone(), 1000)]);

        let tx = Transaction::new(sender.clone(), "bob".to_string(), 100, 1, 0, None);
        let signed = SignedTransaction::sign_with_keypair(&tx, &kp);

        let added = bc.add_block(vec![signed]);
        assert!(added.is_ok());
        assert_eq!(bc.chain.len(), 2);
        assert_eq!(bc.state.get_balance(&sender), 899);
        assert_eq!(bc.state.get_balance("bob"), 100);
    }

    #[test]
    fn rejects_invalid_signature_in_block() {
        let kp = generate_ed25519_keypair();
        let sender = pubkey_to_address_hex(&kp.public);
        let mut bc = Blockchain::with_genesis(vec![(sender.clone(), 1000)]);

        let tx = Transaction::new(sender, "bob".to_string(), 100, 1, 0, None);
        let mut signed = SignedTransaction::sign_with_keypair(&tx, &kp);
        signed.signature = "not-valid".to_string();

        assert!(bc.add_block(vec![signed]).is_err());
#[cfg(test)]
mod tests {
    use super::*;
    use crate::transaction::{
        generate_ed25519_keypair, pubkey_to_address_hex, SignedTransaction, Transaction,
    };

    fn make_signed_tx(
        sender_key: &crate::transaction::KeyPair,
        receiver: &str,
        amount: u64,
        fee: u64,
        nonce: u64,
    ) -> SignedTransaction {
        let sender = pubkey_to_address_hex(&sender_key.public);
        let tx = Transaction::new(sender, receiver.to_string(), amount, fee, nonce, None);
        SignedTransaction::sign_with_keypair(&tx, sender_key)
    }

    #[test]
    fn valid_block_with_valid_tx_list() {
        let sender_kp = generate_ed25519_keypair();
        let sender = pubkey_to_address_hex(&sender_kp.public);

        let mut bc = Blockchain::with_genesis_state(vec![(sender.clone(), 1_000)]);
        let tx = make_signed_tx(&sender_kp, "receiver", 100, 1, 0);
        let block = Block::new(1, vec![tx], bc.last_block().hash.clone());

        assert!(bc.validate_and_add_block(block).is_ok());
        assert_eq!(bc.chain.len(), 2);
        assert_eq!(bc.state.get_balance(&sender), 899);
        assert_eq!(bc.state.get_balance("receiver"), 100);
        assert_eq!(bc.state.get_nonce(&sender), 1);
    }

    #[test]
    fn block_rejected_on_invalid_signature() {
        let sender_kp = generate_ed25519_keypair();
        let sender = pubkey_to_address_hex(&sender_kp.public);

        let mut bc = Blockchain::with_genesis_state(vec![(sender, 1_000)]);
        let mut tx = make_signed_tx(&sender_kp, "receiver", 100, 1, 0);
        tx.signature = "definitely-not-valid-base64-signature".to_string();

        let block = Block::new(1, vec![tx], bc.last_block().hash.clone());
        let result = bc.validate_and_add_block(block);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Invalid transaction in block: InvalidSignature"));
        assert_eq!(bc.chain.len(), 1);
    }

    #[test]
    fn rejects_invalid_nonce_in_block() {
        let kp = generate_ed25519_keypair();
        let sender = pubkey_to_address_hex(&kp.public);
        let mut bc = Blockchain::with_genesis(vec![(sender.clone(), 1000)]);

        let tx = Transaction::new(sender, "bob".to_string(), 100, 1, 5, None);
        let signed = SignedTransaction::sign_with_keypair(&tx, &kp);

        assert!(bc.add_block(vec![signed]).is_err());
    }

    #[test]
    fn rejects_insufficient_balance_in_block() {
        let kp = generate_ed25519_keypair();
        let sender = pubkey_to_address_hex(&kp.public);
        let mut bc = Blockchain::with_genesis(vec![(sender.clone(), 100)]);

        let tx = Transaction::new(sender, "bob".to_string(), 100, 10, 0, None);
        let signed = SignedTransaction::sign_with_keypair(&tx, &kp);

        assert!(bc.add_block(vec![signed]).is_err());
    fn block_rejected_on_nonce_mismatch() {
        let sender_kp = generate_ed25519_keypair();
        let sender = pubkey_to_address_hex(&sender_kp.public);

        let mut bc = Blockchain::with_genesis_state(vec![(sender, 1_000)]);
        let tx = make_signed_tx(&sender_kp, "receiver", 100, 1, 5);

        let block = Block::new(1, vec![tx], bc.last_block().hash.clone());
        let result = bc.validate_and_add_block(block);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Invalid transaction in block: InvalidNonce"));
        assert_eq!(bc.chain.len(), 1);
    }

    #[test]
    fn block_rejected_on_insufficient_balance() {
        let sender_kp = generate_ed25519_keypair();
        let sender = pubkey_to_address_hex(&sender_kp.public);

        let mut bc = Blockchain::with_genesis_state(vec![(sender, 50)]);
        let tx = make_signed_tx(&sender_kp, "receiver", 100, 1, 0);

        let block = Block::new(1, vec![tx], bc.last_block().hash.clone());
        let result = bc.validate_and_add_block(block);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Invalid transaction in block: InsufficientBalance"));
        assert_eq!(bc.chain.len(), 1);
    }
}
