// src/state.rs

use std::collections::HashMap;
use crate::transaction::{SignedTransaction,Transaction};

/// Errors that can occur during state transitions
#[derive(Debug,Clone)]
pub enum StateError{
    InsufficientBalance,
    InvalidNonce,
    InvalidSignature,
    ZeroAmount,
    SenderNotFound,
}

/// Account state
#[derive(Debug,Clone)]
pub struct Account{
    pub balance:u64,
    pub nonce:u64
}

impl Account{
    pub fn new(balance:u64)->Self{
        Self{balance,nonce:0}
    }
}

/// Global chain state (ledger)
#[derive(Debug,Clone)]
pub struct State{
    ///address -> account
    accounts:HashMap<String,Account>,
}

impl State{
    /// Create empty state
    pub fn new()-> Self{
        Self{
            accounts:HashMap::new(),
        }
    }

    /// Create state with genesis balances
    pub fn with_genesis(genesis:Vec<(String,u64)>)->Self{
        let mut accounts=HashMap::new();
        for (addr,balance) in genesis{
            accounts.insert(addr,Account::new(balance))
        }
        Self{accounts}
    }

    /// Get balance of an address
    pub fn get_balance(&self,address:&str)->u64{
        self.accounts
        .get(address)
        .map(|a| a.balance)
        .unwrap_or(0)
    }

    /// Get nonce of an address
    pub fn get_nonce(&self,address:&str)->u64{
        self.accounts
        .get(address)
        .map(|a| a.nonce)
        .unwrap_or(0)
    }

    /// Validate a signed transaction WITHOUT mutating state
    pub fn validate_transaction(&self,tx:&SignedTransaction)->Result<(),StateError>{
        // cryptographic verification
        tx.verify().map_err(|_| StateError::InvalidSignature)?;
        
        let t:&Transaction=&tx.tx;
        if t.amount==0{
            return Err(StateError::ZeroAmount)
        }
        let sender=self
        .accounts
        .get(&t.sender)
        .ok_or(StateError::SenderNotFound)?;

        // nonce check
        if t.nonce!=sender.nonce{
            return Err(StateError::InvalidNonce)
        }

        // balance check (amount + fee)
        let required=t.amount + t.fee;
        if sender.balance<required{
            return Err(StateError::InsufficientBalance)
        }

        Ok(())
    }
    

    /// Apply a signed transaction (Mutates state)
    pub fn apply_transaction(&mut self,tx:&SignedTransaction)->Result<(),StateError>{
        self.validate_transaction(tx)?;

        let t=&tx.tx;
        // subtract from sender
        let sender=self
        .accounts
        .get_mut(&t.sender)
        .expect("Sender must exist after validation");
        sender.balance-=t.amount+t.fee;
        sender.nonce+=1;

        // add to receiver
        let receiver=self
        .account
        .entry(t.receiver.clone())
        .or_insert(Account::new(0));
        receiver.balance+=t.amount;
        // Note: fee handling (burn / validator reward) happens at block level
        Ok(())
    }

    /// Apply multiple transactions atomically (used for blocks)
    pub fn apply_transaction(&mut self,txs:&[SignedTransaction],)->Result<(),StateError>{
        for tx in txs{
            self.apply_transaction(tx)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests[
    use super::*;
    use crate::transaction::{generate_ed25519_keypair,pubkey_to_address_hex,SignedTransaction};

    #[test]
    fn test_basic_transfer(){
        let kp=generate_ed25519_keypair();
        let sender_addr=pubkey_to_address_hex(&kp.public);

        let mut state=State::with_genesis(vec![(sender_addr.clone(),1000)]);

        let tx=Transaction::new(
            sender_addr.clone(),
            "receiver".to_string(),
            100,
            1,
            0,
            None,
        );

        let signed=SignedTransaction::sign_with_keypair(&tx,&kp);

        assert!(state.validate_transaction(&signed).is_ok());
        assert!(state.apply_transaction(&signed).is_ok());

        assert_eq!(state.get_balance(&sender_addr),899);
        assert_eq!(state.get_balance("receiver"),100);
        assert_eq!(state.get_nonce(&sender_addr),1);
    }

    #[test]
    fn test_invalid_nonce(){
        let kp=generate_ed25519_keypair();
        let addr=pubkey_to_address_hex(&kp.public);

        let mut state=State::with_genesis(vec![(addr.clone(),1000)]);

        let tx=Transaction::new(
            addr.clone(),
            "receiver".to_string(),
            100,
            1,
            5, // Wrong nonce
            None,
        );

        let signed=SignedTransaction::sign_with_keypair(&tx,&kp);
        assert!(matches!(
            state.validate_transaction(&signed),
            Err(StateError::InvalidNonce)
        ))
    }

]