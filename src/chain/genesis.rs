// src/chain/genesis.rs
//! Genesis specification shared by every node of a network.
//!
//! All consensus-relevant starting conditions (initial balances, initial validator stakes,
//! chain parameters, genesis time) live in one JSON file. Its SHA-256 is committed into the
//! genesis block (`previous_hash` of block 0), so nodes with a different genesis file end up
//! with a different genesis hash and will never accept each other's blocks.

use anyhow::{anyhow, bail, Context, Result};
use chrono::{TimeZone, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

use crate::block::Block;
use crate::state::{ChainParams, StakePosition, State};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GenesisAllocation {
    pub address: String,
    pub balance: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GenesisValidator {
    pub address: String,
    pub stake: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisSpec {
    /// Human-readable network name, e.g. `netchain-testnet-1`.
    pub chain_id: String,
    /// Genesis time (unix seconds).
    pub timestamp: i64,
    /// Liquid balances at genesis (e.g. the faucet account).
    #[serde(default)]
    pub allocations: Vec<GenesisAllocation>,
    /// Bonded stake at genesis. At least one validator is required to produce block 1.
    #[serde(default)]
    pub validators: Vec<GenesisValidator>,
    /// Initial chain parameters (later changeable through governance).
    #[serde(default)]
    pub params: ChainParams,
}

impl GenesisSpec {
    /// Single-validator development network: `validator` holds the stake and a large balance
    /// that the node's faucet can hand out.
    pub fn dev(validator: &str, timestamp: i64, params: ChainParams) -> Self {
        let stake = params.min_validator_stake.max(1_000);
        Self {
            chain_id: "netchain-dev".to_string(),
            timestamp,
            allocations: vec![GenesisAllocation {
                address: validator.to_string(),
                balance: 1_000_000,
            }],
            validators: vec![GenesisValidator {
                address: validator.to_string(),
                stake,
            }],
            params,
        }
    }

    pub fn load(path: &Path) -> Result<Self> {
        let contents = fs::read_to_string(path)
            .with_context(|| format!("failed to read genesis file {}", path.display()))?;
        Self::from_json(&contents)
            .with_context(|| format!("invalid genesis file {}", path.display()))
    }

    /// Parse and validate a genesis spec from JSON text.
    pub fn from_json(json: &str) -> Result<Self> {
        let spec: Self = serde_json::from_str(json).context("genesis is not valid JSON")?;
        spec.validate()?;
        Ok(spec)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json + "\n")
            .with_context(|| format!("failed to write genesis file {}", path.display()))
    }

    pub fn validate(&self) -> Result<()> {
        if self.chain_id.trim().is_empty() {
            bail!("genesis chain_id must not be empty");
        }
        if self.timestamp < 0 {
            bail!("genesis timestamp must be after the unix epoch");
        }
        if self.validators.is_empty() {
            bail!("genesis must define at least one validator");
        }
        if self.params.block_interval_secs == 0 {
            bail!("block_interval_secs must be > 0");
        }

        let mut supply: u64 = 0;
        let mut seen_validators = HashSet::new();
        for v in &self.validators {
            check_address(&v.address)?;
            if !seen_validators.insert(v.address.as_str()) {
                bail!("duplicate genesis validator {}", v.address);
            }
            if v.stake < self.params.min_validator_stake {
                bail!(
                    "genesis validator {} stake {} is below min_validator_stake {}",
                    v.address,
                    v.stake,
                    self.params.min_validator_stake
                );
            }
            supply = supply
                .checked_add(v.stake)
                .ok_or_else(|| anyhow!("genesis supply overflows u64"))?;
        }

        let mut seen_allocations = HashSet::new();
        for a in &self.allocations {
            check_address(&a.address)?;
            if !seen_allocations.insert(a.address.as_str()) {
                bail!("duplicate genesis allocation {}", a.address);
            }
            supply = supply
                .checked_add(a.balance)
                .ok_or_else(|| anyhow!("genesis supply overflows u64"))?;
        }
        Ok(())
    }

    /// SHA-256 of the canonical JSON encoding. Committed into the genesis block.
    pub fn hash(&self) -> String {
        let bytes = serde_json::to_vec(self).expect("genesis spec serializes");
        hex::encode(Sha256::digest(&bytes))
    }

    pub fn build_state(&self) -> State {
        let mut state = State::with_genesis(
            self.allocations
                .iter()
                .map(|a| (a.address.clone(), a.balance))
                .collect(),
        );
        for v in &self.validators {
            state.accounts.entry(v.address.clone()).or_default();
            state
                .stakes
                .insert(v.address.clone(), StakePosition { amount: v.stake });
        }
        state.chain_params = self.params.clone();
        state
    }

    pub fn genesis_block(&self) -> Block {
        let ts = Utc
            .timestamp_opt(self.timestamp, 0)
            .single()
            .expect("validated genesis timestamp");
        Block::genesis(self.hash(), ts)
    }
}

fn check_address(address: &str) -> Result<()> {
    if address.len() != 40 || hex::decode(address).is_err() {
        bail!("invalid address {address:?}: expected 40 hex characters");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const ADDR: &str = "0123456789abcdef0123456789abcdef01234567";

    #[test]
    fn dev_genesis_is_valid_and_deterministic() {
        let spec = GenesisSpec::dev(ADDR, 1_700_000_000, ChainParams::default());
        spec.validate().unwrap();
        assert_eq!(spec.hash(), spec.clone().hash());
        assert_eq!(spec.genesis_block().hash, spec.genesis_block().hash);

        let state = spec.build_state();
        assert_eq!(state.get_balance(ADDR), 1_000_000);
        assert!(state.get_staked_balance(ADDR) >= 1_000);
    }

    #[test]
    fn different_specs_have_different_genesis_blocks() {
        let a = GenesisSpec::dev(ADDR, 1_700_000_000, ChainParams::default());
        let mut b = a.clone();
        b.chain_id = "other".into();
        assert_ne!(a.genesis_block().hash, b.genesis_block().hash);
    }

    #[test]
    fn rejects_invalid_specs() {
        let mut spec = GenesisSpec::dev(ADDR, 1_700_000_000, ChainParams::default());
        spec.validators.clear();
        assert!(spec.validate().is_err());

        let mut spec = GenesisSpec::dev(ADDR, 1_700_000_000, ChainParams::default());
        spec.validators[0].address = "genesis_account".into();
        assert!(spec.validate().is_err());

        let mut spec = GenesisSpec::dev(ADDR, 1_700_000_000, ChainParams::default());
        spec.validators[0].stake = 1;
        assert!(spec.validate().is_err());
    }
}
