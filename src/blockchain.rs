// src/blockchain.rs

use crate::block::Block;

#[derive(Debug)]
pub struct Blockchain {
    pub chain: Vec<Block>,
}

impl Blockchain {
    pub fn new() -> Self {
        let mut bc = Blockchain { chain: Vec::new() };
        bc.chain.push(Self::genesis_block());
        bc
    }

    fn genesis_block() -> Block {
        Block::new(0, "Genesis Block".to_string(), "0".to_string())
    }

    pub fn last_block(&self) -> &Block {
        self.chain
            .last()
            .expect("Blockchain must have at least one block")
    }

    /// Used by local miner / validator
    pub fn add_block(&mut self, data: String) -> Block {
        let last = self.last_block();
        let new_block = Block::new(last.index + 1, data, last.hash.clone());
        self.chain.push(new_block.clone());
        new_block
    }

    /// Used when receiving blocks from P2P
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
            &block.data,
            &block.previous_hash,
        );

        if block.hash != recalculated {
            return Err("Invalid block hash".into());
        }

        self.chain.push(block);
        Ok(())
    }

    pub fn is_valid(&self) -> bool {
        for i in 1..self.chain.len() {
            let current = &self.chain[i];
            let previous = &self.chain[i - 1];

            if current.previous_hash != previous.hash {
                return false;
            }

            let recalculated = Block::calculate_hash(
                current.index,
                &current.timestamp,
                &current.data,
                &current.previous_hash,
            );

            if current.hash != recalculated {
                return false;
            }
        }
        true
    }
}
