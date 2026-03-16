// src/chain/state.rs

use crate::transaction::{ProposalAction, SignedTransaction, Transaction, TransactionType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::info;

/// Errors that can occur during state transitions
#[derive(Debug, Clone)]
pub enum StateError {
    InsufficientBalance,
    InvalidNonce,
    InvalidSignature,
    ZeroAmount,
    SenderNotFound,
    InsufficientStake,
    ProposalNotFound,
    ProposalExpired,
    DuplicateVote,
    NotEnoughStakeToPropose,
    InvalidProposalAction,
}

/// Account state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub balance: u64,
    pub nonce: u64,
}

impl Account {
    pub fn new(balance: u64) -> Self {
        Self { balance, nonce: 0 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StakePosition {
    pub amount: u64,
}

/// Runtime-mutable chain parameters that can be changed via governance
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ChainParams {
    /// Tokens minted per block (block reward)
    pub block_reward: u64,
    /// Block production interval in seconds
    pub block_interval_secs: u64,
    /// Maximum transactions per block
    pub max_txs_per_block: usize,
    /// Stake weight for validator selection (0.0 = pure PoI, 1.0 = pure stake)
    pub stake_weight: f64,
    /// Minimum share of total stake that must participate for a proposal to pass.
    pub proposal_quorum_bps: u64,
    /// Minimum yes-vote share of participating stake required to pass.
    pub proposal_approval_bps: u64,
    /// Minimum stake required to create a proposal.
    pub min_proposal_stake: u64,
}

impl Default for ChainParams {
    fn default() -> Self {
        Self {
            block_reward: 50,
            block_interval_secs: 15,
            max_txs_per_block: 100,
            stake_weight: 0.3,
            proposal_quorum_bps: 2_000,
            proposal_approval_bps: 5_001,
            min_proposal_stake: 100,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GovernanceProposal {
    pub id: u64,
    pub proposer: String,
    pub title: String,
    pub description: String,
    pub created_at: u64,
    pub expires_at: u64,
    pub yes_votes: u64,
    pub no_votes: u64,
    pub voters: HashMap<String, bool>,
    /// Optional action to execute if the proposal passes
    pub action: Option<ProposalAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProposalStatus {
    Active,
    Passed,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExecutedProposal {
    pub proposal_id: u64,
    pub title: String,
    pub yes_votes: u64,
    pub no_votes: u64,
    pub action: ProposalAction,
}

/// Events emitted during state transitions that callers can broadcast via WebSocket.
#[derive(Debug, Clone)]
pub enum StateEvent {
    /// A new governance proposal was created.
    ProposalCreated {
        proposal_id: u64,
        title: String,
        proposer: String,
    },
    /// A vote was cast on a governance proposal.
    VoteCast {
        proposal_id: u64,
        title: String,
        voter: String,
        support: bool,
        yes_votes: u64,
        no_votes: u64,
    },
}

impl GovernanceProposal {
    pub fn total_votes(&self) -> u64 {
        self.yes_votes + self.no_votes
    }

    pub fn status(
        &self,
        now: u64,
        chain_params: &ChainParams,
        total_staked: u64,
    ) -> ProposalStatus {
        if now < self.expires_at {
            return ProposalStatus::Active;
        }

        let total_votes = self.total_votes();
        if total_votes == 0 || total_staked == 0 {
            return ProposalStatus::Rejected;
        }

        let quorum_votes = total_staked.saturating_mul(chain_params.proposal_quorum_bps) / 10_000;
        if total_votes < quorum_votes.max(1) {
            return ProposalStatus::Rejected;
        }

        let yes_bps = self.yes_votes.saturating_mul(10_000) / total_votes;
        if yes_bps < chain_params.proposal_approval_bps || self.yes_votes <= self.no_votes {
            return ProposalStatus::Rejected;
        }

        ProposalStatus::Passed
    }
}

/// Global chain state (ledger)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    ///address -> account
    pub accounts: HashMap<String, Account>,
    pub stakes: HashMap<String, StakePosition>,
    pub proposals: HashMap<u64, GovernanceProposal>,
    pub next_proposal_id: u64,
    /// Runtime-mutable chain parameters
    #[serde(default)]
    pub chain_params: ChainParams,
}

impl State {
    /// Create empty state
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
            stakes: HashMap::new(),
            proposals: HashMap::new(),
            next_proposal_id: 1,
            chain_params: ChainParams::default(),
        }
    }

    /// Create state with genesis balances
    pub fn with_genesis(genesis: Vec<(String, u64)>) -> Self {
        let mut accounts = HashMap::new();
        for (addr, balance) in genesis {
            accounts.insert(addr, Account::new(balance));
        }
        Self {
            accounts,
            stakes: HashMap::new(),
            proposals: HashMap::new(),
            next_proposal_id: 1,
            chain_params: ChainParams::default(),
        }
    }

    /// Create state from existing accounts (for loading from storage)
    pub fn from_accounts(accounts: HashMap<String, Account>) -> Self {
        Self {
            accounts,
            stakes: HashMap::new(),
            proposals: HashMap::new(),
            next_proposal_id: 1,
            chain_params: ChainParams::default(),
        }
    }

    /// Get reference to all accounts (for saving to storage)
    pub fn get_accounts(&self) -> &HashMap<String, Account> {
        &self.accounts
    }

    pub fn get_staked_balance(&self, address: &str) -> u64 {
        self.stakes.get(address).map(|s| s.amount).unwrap_or(0)
    }

    pub fn total_staked(&self) -> u64 {
        self.stakes.values().map(|s| s.amount).sum()
    }

    /// Get a map of address -> staked amount for all stakers (used for validator selection)
    pub fn get_stake_map(&self) -> HashMap<String, u64> {
        self.stakes
            .iter()
            .filter(|(_, s)| s.amount > 0)
            .map(|(addr, s)| (addr.clone(), s.amount))
            .collect()
    }

    pub fn get_proposal(&self, proposal_id: u64) -> Option<&GovernanceProposal> {
        self.proposals.get(&proposal_id)
    }

    pub fn list_proposals(&self) -> Vec<&GovernanceProposal> {
        let mut proposals: Vec<&GovernanceProposal> = self.proposals.values().collect();
        proposals.sort_by_key(|proposal| proposal.id);
        proposals
    }

    pub fn proposal_status(&self, proposal: &GovernanceProposal, now: u64) -> ProposalStatus {
        proposal.status(now, &self.chain_params, self.total_staked())
    }

    /// Get balance of an address
    pub fn get_balance(&self, address: &str) -> u64 {
        self.accounts.get(address).map(|a| a.balance).unwrap_or(0)
    }

    /// Get nonce of an address
    pub fn get_nonce(&self, address: &str) -> u64 {
        self.accounts.get(address).map(|a| a.nonce).unwrap_or(0)
    }

    /// Validate a signed transaction WITHOUT mutating state
    pub fn validate_transaction(&self, tx: &SignedTransaction) -> Result<(), StateError> {
        let now = current_unix_timestamp();
        self.validate_transaction_at(tx, now)
    }

    /// Validate a signed transaction at a deterministic "chain time" (seconds since epoch).
    ///
    /// Use this for block processing so all nodes evaluate time-based rules identically.
    pub fn validate_transaction_at(
        &self,
        tx: &SignedTransaction,
        now: u64,
    ) -> Result<(), StateError> {
        // cryptographic verification
        tx.verify().map_err(|_| StateError::InvalidSignature)?;

        let t: &Transaction = &tx.tx;
        let sender = self
            .accounts
            .get(&t.sender)
            .ok_or(StateError::SenderNotFound)?;

        // nonce check
        if t.nonce != sender.nonce {
            return Err(StateError::InvalidNonce);
        }

        // balance check (amount + fee)
        let required = t.amount + t.fee;
        if sender.balance < required {
            return Err(StateError::InsufficientBalance);
        }

        match &t.tx_type {
            TransactionType::Transfer => {
                if t.amount == 0 {
                    return Err(StateError::ZeroAmount);
                }
                if t.receiver.is_empty() {
                    return Err(StateError::SenderNotFound);
                }
            }
            TransactionType::Stake => {
                if t.amount == 0 {
                    return Err(StateError::ZeroAmount);
                }
            }
            TransactionType::Unstake => {
                if t.amount == 0 {
                    return Err(StateError::ZeroAmount);
                }
                if self.get_staked_balance(&t.sender) < t.amount {
                    return Err(StateError::InsufficientStake);
                }
            }
            TransactionType::CreateProposal {
                title,
                description,
                voting_period_secs,
                action,
            } => {
                if title.trim().is_empty()
                    || description.trim().is_empty()
                    || *voting_period_secs == 0
                {
                    return Err(StateError::ZeroAmount);
                }
                if self.get_staked_balance(&t.sender) < self.chain_params.min_proposal_stake {
                    return Err(StateError::NotEnoughStakeToPropose);
                }
                if let Some(action) = action {
                    self.validate_proposal_action(action)?;
                }
            }
            TransactionType::VoteProposal {
                proposal_id,
                support: _,
            } => {
                let proposal = self
                    .proposals
                    .get(proposal_id)
                    .ok_or(StateError::ProposalNotFound)?;
                if self.proposal_status(proposal, now) != ProposalStatus::Active {
                    return Err(StateError::ProposalExpired);
                }
                if proposal.voters.contains_key(&t.sender) {
                    return Err(StateError::DuplicateVote);
                }
                if self.get_staked_balance(&t.sender) == 0 {
                    return Err(StateError::InsufficientStake);
                }
            }
        }

        Ok(())
    }

    fn validate_proposal_action(&self, action: &ProposalAction) -> Result<(), StateError> {
        match action {
            ProposalAction::ChangeBlockReward(new_reward) => {
                if *new_reward == 0 || *new_reward > 1_000_000 {
                    return Err(StateError::InvalidProposalAction);
                }
            }
            ProposalAction::ChangeBlockInterval(new_interval) => {
                if *new_interval == 0 || *new_interval > 3_600 {
                    return Err(StateError::InvalidProposalAction);
                }
            }
            ProposalAction::ChangeMaxTxsPerBlock(new_max) => {
                if *new_max == 0 || *new_max > 10_000 {
                    return Err(StateError::InvalidProposalAction);
                }
            }
            ProposalAction::ChangeStakeWeight(basis_points) => {
                if *basis_points > 10_000 {
                    return Err(StateError::InvalidProposalAction);
                }
            }
        }

        Ok(())
    }

    /// Apply a signed transaction (Mutates state)
    pub fn apply_transaction(
        &mut self,
        tx: &SignedTransaction,
    ) -> Result<Option<StateEvent>, StateError> {
        let now = current_unix_timestamp();
        self.apply_transaction_at(tx, now)
    }

    /// Apply a signed transaction at a deterministic "chain time" (seconds since epoch).
    ///
    /// IMPORTANT: For consensus-critical paths (block execution), pass the block timestamp.
    /// Returns an optional `StateEvent` that callers can broadcast via WebSocket.
    pub fn apply_transaction_at(
        &mut self,
        tx: &SignedTransaction,
        now: u64,
    ) -> Result<Option<StateEvent>, StateError> {
        self.validate_transaction_at(tx, now)?;

        let t = &tx.tx;
        // subtract fees and maybe principal from sender
        let sender = self
            .accounts
            .get_mut(&t.sender)
            .expect("Sender must exist after validation");
        sender.balance -= t.amount + t.fee;
        sender.nonce += 1;

        let event = match &t.tx_type {
            TransactionType::Transfer => {
                let receiver = self
                    .accounts
                    .entry(t.receiver.clone())
                    .or_insert(Account::new(0));
                receiver.balance += t.amount;
                None
            }
            TransactionType::Stake => {
                let stake = self
                    .stakes
                    .entry(t.sender.clone())
                    .or_insert(StakePosition { amount: 0 });
                stake.amount += t.amount;
                None
            }
            TransactionType::Unstake => {
                let stake = self
                    .stakes
                    .entry(t.sender.clone())
                    .or_insert(StakePosition { amount: 0 });
                stake.amount -= t.amount;
                let sender = self
                    .accounts
                    .get_mut(&t.sender)
                    .expect("Sender must exist after validation");
                sender.balance += t.amount;
                None
            }
            TransactionType::CreateProposal {
                title,
                description,
                voting_period_secs,
                action,
            } => {
                let created_at = now;
                let proposal_id = self.next_proposal_id;
                self.next_proposal_id += 1;
                self.proposals.insert(
                    proposal_id,
                    GovernanceProposal {
                        id: proposal_id,
                        proposer: t.sender.clone(),
                        title: title.clone(),
                        description: description.clone(),
                        created_at,
                        expires_at: created_at + voting_period_secs,
                        yes_votes: 0,
                        no_votes: 0,
                        voters: HashMap::new(),
                        action: action.clone(),
                    },
                );
                Some(StateEvent::ProposalCreated {
                    proposal_id,
                    title: title.clone(),
                    proposer: t.sender.clone(),
                })
            }
            TransactionType::VoteProposal {
                proposal_id,
                support,
            } => {
                let voting_power = self.get_staked_balance(&t.sender);
                let proposal = self
                    .proposals
                    .get_mut(proposal_id)
                    .expect("Proposal must exist after validation");
                proposal.voters.insert(t.sender.clone(), *support);
                if *support {
                    proposal.yes_votes += voting_power;
                } else {
                    proposal.no_votes += voting_power;
                }
                Some(StateEvent::VoteCast {
                    proposal_id: *proposal_id,
                    title: proposal.title.clone(),
                    voter: t.sender.clone(),
                    support: *support,
                    yes_votes: proposal.yes_votes,
                    no_votes: proposal.no_votes,
                })
            }
        };

        // Note: fee handling (burn / validator reward) happens at block level
        Ok(event)
    }

    /// Apply multiple transactions atomically (used for blocks)
    pub fn apply_transactions(
        &mut self,
        txs: &[SignedTransaction],
    ) -> Result<Vec<StateEvent>, StateError> {
        let now = current_unix_timestamp();
        self.apply_transactions_at(txs, now)
    }

    /// Apply multiple transactions atomically at a deterministic "chain time".
    pub fn apply_transactions_at(
        &mut self,
        txs: &[SignedTransaction],
        now: u64,
    ) -> Result<Vec<StateEvent>, StateError> {
        let mut events = Vec::new();
        for tx in txs {
            if let Some(event) = self.apply_transaction_at(tx, now)? {
                events.push(event);
            }
        }
        Ok(events)
    }

    /// Distribute block rewards to the validator:
    /// - Sum of all transaction fees in the block
    /// - A fixed block reward (new token issuance / inflation)
    ///
    /// The validator account is created if it doesn't exist.
    pub fn apply_block_rewards(
        &mut self,
        validator_address: &str,
        transactions: &[SignedTransaction],
        block_reward: u64,
    ) {
        let total_fees: u64 = transactions.iter().map(|tx| tx.tx.fee).sum();
        let total_reward = total_fees + block_reward;

        if total_reward > 0 {
            let validator = self
                .accounts
                .entry(validator_address.to_string())
                .or_insert(Account::new(0));
            validator.balance += total_reward;
        }
    }

    /// Execute all passed proposals that have expired.
    /// Returns the list of executed proposal IDs for event broadcasting.
    /// This should be called periodically (e.g., after each block) to apply governance decisions.
    pub fn execute_passed_proposals(&mut self) -> Vec<ExecutedProposal> {
        let now = current_unix_timestamp();
        self.execute_passed_proposals_at(now)
    }

    /// Execute all passed proposals that have expired at a deterministic "chain time".
    pub fn execute_passed_proposals_at(&mut self, now: u64) -> Vec<ExecutedProposal> {
        let mut executed_proposals = Vec::new();

        // Collect proposals that need execution
        let proposals_to_execute: Vec<ExecutedProposal> = self
            .proposals
            .iter()
            .filter_map(|(id, proposal)| {
                if self.proposal_status(proposal, now) == ProposalStatus::Passed {
                    proposal.action.clone().map(|action| ExecutedProposal {
                        proposal_id: *id,
                        title: proposal.title.clone(),
                        yes_votes: proposal.yes_votes,
                        no_votes: proposal.no_votes,
                        action,
                    })
                } else {
                    None
                }
            })
            .collect();

        // Execute each proposal
        for executed in &proposals_to_execute {
            match &executed.action {
                ProposalAction::ChangeBlockReward(new_reward) => {
                    self.chain_params.block_reward = *new_reward;
                    info!(
                        "Executed proposal {}: changed block reward to {}",
                        executed.proposal_id, new_reward
                    );
                }
                ProposalAction::ChangeBlockInterval(new_interval) => {
                    self.chain_params.block_interval_secs = *new_interval;
                    info!(
                        "Executed proposal {}: changed block interval to {}s",
                        executed.proposal_id, new_interval
                    );
                }
                ProposalAction::ChangeMaxTxsPerBlock(new_max) => {
                    self.chain_params.max_txs_per_block = *new_max;
                    info!(
                        "Executed proposal {}: changed max txs per block to {}",
                        executed.proposal_id, new_max
                    );
                }
                ProposalAction::ChangeStakeWeight(basis_points) => {
                    // Convert basis points to decimal (e.g., 3000 -> 0.3)
                    self.chain_params.stake_weight = (*basis_points as f64) / 10_000.0;
                    info!(
                        "Executed proposal {}: changed stake weight to {}",
                        executed.proposal_id, self.chain_params.stake_weight
                    );
                }
            }
            executed_proposals.push(executed.clone());
        }

        // Remove executed proposals from state
        for executed in &executed_proposals {
            self.proposals.remove(&executed.proposal_id);
        }

        executed_proposals
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
    use crate::transaction::{
        generate_ed25519_keypair, pubkey_to_address_hex, SignedTransaction, Transaction,
    };

    #[test]
    fn test_basic_transfer() {
        let kp = generate_ed25519_keypair();
        let sender_addr = pubkey_to_address_hex(&kp.verifying_key());

        let mut state = State::with_genesis(vec![(sender_addr.clone(), 1000)]);

        let tx = Transaction::new(sender_addr.clone(), "receiver".to_string(), 100, 1, 0, None);

        let signed = SignedTransaction::sign_with_keypair(&tx, &kp);

        assert!(state.validate_transaction(&signed).is_ok());
        assert!(state.apply_transaction(&signed).is_ok());

        assert_eq!(state.get_balance(&sender_addr), 899);
        assert_eq!(state.get_balance("receiver"), 100);
        assert_eq!(state.get_nonce(&sender_addr), 1);
    }

    #[test]
    fn test_invalid_nonce() {
        let kp = generate_ed25519_keypair();
        let addr = pubkey_to_address_hex(&kp.verifying_key());

        let state = State::with_genesis(vec![(addr.clone(), 1000)]);

        let tx = Transaction::new(
            addr.clone(),
            "receiver".to_string(),
            100,
            1,
            5, // Wrong nonce
            None,
        );

        let signed = SignedTransaction::sign_with_keypair(&tx, &kp);
        assert!(matches!(
            state.validate_transaction(&signed),
            Err(StateError::InvalidNonce)
        ))
    }

    #[test]
    fn test_block_rewards_distribution() {
        let kp = generate_ed25519_keypair();
        let sender_addr = pubkey_to_address_hex(&kp.verifying_key());
        let validator_addr = "validator_address".to_string();

        let mut state = State::with_genesis(vec![(sender_addr.clone(), 10_000)]);

        // Create two transactions with fees
        let tx1 = Transaction::new(sender_addr.clone(), "alice".to_string(), 100, 5, 0, None);
        let signed1 = SignedTransaction::sign_with_keypair(&tx1, &kp);

        let tx2 = Transaction::new(sender_addr.clone(), "bob".to_string(), 200, 10, 1, None);
        let signed2 = SignedTransaction::sign_with_keypair(&tx2, &kp);

        let txs = vec![signed1.clone(), signed2.clone()];

        // Apply transactions
        state.apply_transaction(&signed1).unwrap();
        state.apply_transaction(&signed2).unwrap();

        // Distribute rewards: fees (5+10=15) + block_reward (50) = 65
        state.apply_block_rewards(&validator_addr, &txs, 50);

        assert_eq!(state.get_balance(&validator_addr), 65);
        // Sender should have lost: 100+5 + 200+10 = 315
        assert_eq!(state.get_balance(&sender_addr), 10_000 - 315);
    }

    #[test]
    fn test_stake_and_unstake() {
        let kp = generate_ed25519_keypair();
        let addr = pubkey_to_address_hex(&kp.verifying_key());
        let mut state = State::with_genesis(vec![(addr.clone(), 1_000)]);

        let stake_tx = Transaction::stake(addr.clone(), 300, 5, 0);
        let signed_stake = SignedTransaction::sign_with_keypair(&stake_tx, &kp);
        state.apply_transaction(&signed_stake).unwrap();

        assert_eq!(state.get_balance(&addr), 695);
        assert_eq!(state.get_staked_balance(&addr), 300);

        let unstake_tx = Transaction::unstake(addr.clone(), 100, 2, 1);
        let signed_unstake = SignedTransaction::sign_with_keypair(&unstake_tx, &kp);
        state.apply_transaction(&signed_unstake).unwrap();

        assert_eq!(state.get_balance(&addr), 693);
        assert_eq!(state.get_staked_balance(&addr), 200);
    }

    #[test]
    fn test_create_and_vote_proposal() {
        let kp1 = generate_ed25519_keypair();
        let kp2 = generate_ed25519_keypair();
        let addr1 = pubkey_to_address_hex(&kp1.verifying_key());
        let addr2 = pubkey_to_address_hex(&kp2.verifying_key());
        let mut state = State::with_genesis(vec![(addr1.clone(), 5_000), (addr2.clone(), 5_000)]);

        let stake1 = SignedTransaction::sign_with_keypair(
            &Transaction::stake(addr1.clone(), 500, 1, 0),
            &kp1,
        );
        let stake2 = SignedTransaction::sign_with_keypair(
            &Transaction::stake(addr2.clone(), 300, 1, 0),
            &kp2,
        );
        state.apply_transaction(&stake1).unwrap();
        state.apply_transaction(&stake2).unwrap();

        let proposal = Transaction::create_proposal(
            addr1.clone(),
            2,
            1,
            "Increase validator rewards".to_string(),
            "Raise block reward for testnet validators".to_string(),
            600,
        );
        let signed_proposal = SignedTransaction::sign_with_keypair(&proposal, &kp1);
        state.apply_transaction(&signed_proposal).unwrap();

        let proposal = state.get_proposal(1).unwrap();
        assert_eq!(proposal.title, "Increase validator rewards");

        let vote = Transaction::vote_proposal(addr2.clone(), 1, 1, 1, true);
        let signed_vote = SignedTransaction::sign_with_keypair(&vote, &kp2);
        state.apply_transaction(&signed_vote).unwrap();

        let proposal = state.get_proposal(1).unwrap();
        assert_eq!(proposal.yes_votes, 300);
        assert_eq!(proposal.no_votes, 0);
    }

    #[test]
    fn test_vote_requires_stake() {
        let kp1 = generate_ed25519_keypair();
        let kp2 = generate_ed25519_keypair();
        let addr1 = pubkey_to_address_hex(&kp1.verifying_key());
        let addr2 = pubkey_to_address_hex(&kp2.verifying_key());
        let mut state = State::with_genesis(vec![(addr1.clone(), 2_000), (addr2.clone(), 2_000)]);

        let stake1 = SignedTransaction::sign_with_keypair(
            &Transaction::stake(addr1.clone(), 500, 1, 0),
            &kp1,
        );
        state.apply_transaction(&stake1).unwrap();

        let proposal = SignedTransaction::sign_with_keypair(
            &Transaction::create_proposal(
                addr1.clone(),
                1,
                1,
                "Test".to_string(),
                "Desc".to_string(),
                600,
            ),
            &kp1,
        );
        state.apply_transaction(&proposal).unwrap();

        let vote = SignedTransaction::sign_with_keypair(
            &Transaction::vote_proposal(addr2.clone(), 1, 0, 1, true),
            &kp2,
        );
        assert!(matches!(
            state.validate_transaction(&vote),
            Err(StateError::InsufficientStake)
        ));
    }

    #[test]
    fn test_create_and_execute_proposal() {
        let kp1 = generate_ed25519_keypair();
        let kp2 = generate_ed25519_keypair();
        let addr1 = pubkey_to_address_hex(&kp1.verifying_key());
        let addr2 = pubkey_to_address_hex(&kp2.verifying_key());
        let mut state = State::with_genesis(vec![(addr1.clone(), 5_000), (addr2.clone(), 5_000)]);

        let stake1 = SignedTransaction::sign_with_keypair(
            &Transaction::stake(addr1.clone(), 500, 1, 0),
            &kp1,
        );
        let stake2 = SignedTransaction::sign_with_keypair(
            &Transaction::stake(addr2.clone(), 300, 1, 0),
            &kp2,
        );
        state.apply_transaction(&stake1).unwrap();
        state.apply_transaction(&stake2).unwrap();

        // Create a proposal to change block reward to 100
        let proposal = Transaction::create_proposal_with_action(
            addr1.clone(),
            2,
            1,
            "Change block reward".to_string(),
            "Increase block reward to 100".to_string(),
            1, // 1 second voting period
            ProposalAction::ChangeBlockReward(100),
        );
        let signed_proposal = SignedTransaction::sign_with_keypair(&proposal, &kp1);
        state.apply_transaction(&signed_proposal).unwrap();

        // Vote yes
        let vote = Transaction::vote_proposal(addr2.clone(), 1, 1, 1, true);
        let signed_vote = SignedTransaction::sign_with_keypair(&vote, &kp2);
        state.apply_transaction(&signed_vote).unwrap();

        // Check that proposal exists
        let prop = state.get_proposal(1).unwrap();
        assert_eq!(prop.yes_votes, 300);

        // Fast-forward time by waiting 2 seconds (proposal should be passed)
        std::thread::sleep(std::time::Duration::from_secs(2));

        // Execute proposals
        let executed = state.execute_passed_proposals();
        assert_eq!(executed.len(), 1);
        assert_eq!(executed[0].proposal_id, 1);
        match executed[0].action {
            ProposalAction::ChangeBlockReward(reward) => assert_eq!(reward, 100),
            _ => panic!("Wrong action"),
        }

        // Check that chain params were updated
        assert_eq!(state.chain_params.block_reward, 100);

        // Proposal should be removed after execution
        assert!(state.get_proposal(1).is_none());
    }

    #[test]
    fn test_stake_weight_change_proposal() {
        let kp = generate_ed25519_keypair();
        let addr = pubkey_to_address_hex(&kp.verifying_key());
        let mut state = State::with_genesis(vec![(addr.clone(), 2_000)]);

        let stake =
            SignedTransaction::sign_with_keypair(&Transaction::stake(addr.clone(), 500, 1, 0), &kp);
        state.apply_transaction(&stake).unwrap();

        // Create proposal to change stake weight to 50% (5000 basis points)
        let proposal = Transaction::create_proposal_with_action(
            addr.clone(),
            2,
            1,
            "Change stake weight".to_string(),
            "Increase stake influence to 50%".to_string(),
            1,
            ProposalAction::ChangeStakeWeight(5000),
        );
        let signed_proposal = SignedTransaction::sign_with_keypair(&proposal, &kp);
        state.apply_transaction(&signed_proposal).unwrap();

        // Vote yes
        let vote = Transaction::vote_proposal(addr.clone(), 1, 2, 1, true);
        let signed_vote = SignedTransaction::sign_with_keypair(&vote, &kp);
        state.apply_transaction(&signed_vote).unwrap();

        // Wait for proposal to expire
        std::thread::sleep(std::time::Duration::from_secs(2));

        // Execute
        let executed = state.execute_passed_proposals();
        assert_eq!(executed.len(), 1);

        // Check stake weight changed to 0.5
        assert!((state.chain_params.stake_weight - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_invalid_proposal_action_rejected() {
        let kp = generate_ed25519_keypair();
        let addr = pubkey_to_address_hex(&kp.verifying_key());
        let mut state = State::with_genesis(vec![(addr.clone(), 5_000)]);

        let stake =
            SignedTransaction::sign_with_keypair(&Transaction::stake(addr.clone(), 500, 1, 0), &kp);
        state.apply_transaction(&stake).unwrap();

        let proposal = SignedTransaction::sign_with_keypair(
            &Transaction::create_proposal_with_action(
                addr.clone(),
                1,
                1,
                "Bad stake weight".to_string(),
                "Should fail validation".to_string(),
                60,
                ProposalAction::ChangeStakeWeight(10_001),
            ),
            &kp,
        );

        assert!(matches!(
            state.validate_transaction(&proposal),
            Err(StateError::InvalidProposalAction)
        ));
    }

    #[test]
    fn test_proposal_requires_quorum_to_pass() {
        let kp1 = generate_ed25519_keypair();
        let kp2 = generate_ed25519_keypair();
        let addr1 = pubkey_to_address_hex(&kp1.verifying_key());
        let addr2 = pubkey_to_address_hex(&kp2.verifying_key());
        let mut state = State::with_genesis(vec![(addr1.clone(), 10_000), (addr2.clone(), 10_000)]);

        let stake1 = SignedTransaction::sign_with_keypair(
            &Transaction::stake(addr1.clone(), 500, 1, 0),
            &kp1,
        );
        let stake2 = SignedTransaction::sign_with_keypair(
            &Transaction::stake(addr2.clone(), 9_000, 1, 0),
            &kp2,
        );
        state.apply_transaction(&stake1).unwrap();
        state.apply_transaction(&stake2).unwrap();

        let proposal = SignedTransaction::sign_with_keypair(
            &Transaction::create_proposal_with_action(
                addr1.clone(),
                1,
                1,
                "Increase rewards".to_string(),
                "Small holder tries to change rewards".to_string(),
                1,
                ProposalAction::ChangeBlockReward(100),
            ),
            &kp1,
        );
        state.apply_transaction(&proposal).unwrap();

        let now = current_unix_timestamp() + 2;
        let proposal = state.get_proposal(1).unwrap();
        assert_eq!(
            state.proposal_status(proposal, now),
            ProposalStatus::Rejected
        );

        let executed = state.execute_passed_proposals_at(now);
        assert!(executed.is_empty());
        assert_eq!(state.chain_params.block_reward, 50);
        assert!(state.get_proposal(1).is_some());
    }
}
