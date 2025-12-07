use chrono::{DateTime,Utc};
use serde::{Deserialize,Serialize};
use sha2::{Digest,Sha256};


#[derive(Serialize,Deserialize,Debug,Clone)]
struct Block{
    index:u64,
    timestamp:DateTime<Utc>,
    data:String,
    previous_hash:String,
    hash:String,
}

impl Block{
    fn new(index:u64,data:String,previous_hash:String)->Self{
        let timestamp=Utc::now();
        let hash=Block::calculate_hash(index,&timestamp,&data,&previous_hash);
        Block{
            index,
            timestamp,
            data,
            previous_hash,
            hash,
        }
    }

    fn calculate_hash(index:u64,timestamp:&DateTime<Utc>,data:&str,previous_hash:&str)->String{
        // Simple hash over fields (Json encoding)
        let payload=serde_json::json!({
            "index":index,
            "timestamp":timestamp.to_rfc3339(),
            "data":data,
            "previous_hash":previous_hash,
        })
        .to_string();

        let mut hasher=Sha256::new();
        hasher.update(payload.as_bytes());
        let result=hasher.finalize();
        format!("{:x}",result)
    }
}

struct Blockchain{
    chain:Vec<Block>,
}

impl Blockchain{
    fn new()->Self{
        let mut bc=Blockchain{
            chain:Vec::new()
        };
        let genesis=Blockchain::genesis_block();
        bc.chain.push(genesis);
        bc
    }
    fn genesis_block()->Block{
        //The first block -index 0
        Block::new(0,"Genesis Block".to_string(),"0".to_string())
    }

    fn last_block(&self)->&Block{
        self.chain.last().expect("Blockchain must have at least one block")
    }

    fn add_block(&mut self,data:String){
        let last=self.last_block();
        let new_index=last.index+1;
        let new_block=Block::new(new_index,data,last.hash.clone());
        self.chain.push(new_block);
    }

    fn is_valid(&self)->bool{
        //Validate chain: hashes and linkage
        for i in 1..self.chain.len(){
            let current=&self.chain[i];
            let previous=&self.chain[i-1];

            //Check previous hash reference
            if current.previous_hash!=previous.hash{
                eprintln!(
                    "Invalid chain: block {} previous_hash mismatch",
                    current.index
                );
                return false;
            }

            // Recalculate hash and compare
            let recalculated=Block::calculate_hash(
                current.index,
                &current.timestamp,
                &current.data,
                &current.previous_hash,
            );
            if current.hash!=recalculated{
                eprintln!("Invalid chain: block {} has invalid hash",current.index);
                return false;
            }
        }
        true
    }
}

fn main(){
    println!("Starting NetChain (developement mode)\n");
    
    let mut chain=Blockchain::new();
    println!("Genesis: {:?}",chain.last_block());

    //Add a few blocks
    chain.add_block("Alice pays Bob 10NC".to_string());
    chain.add_block("Bob pays Clara 5NC".to_string());
    chain.add_block("Clara stakes 50NC".to_string());

    println!("\nChains:");
    for block in &chain.chain{
        println!(
            "Index: {}, Time: {}, Date: {}, Hash: {}",
            block.index,
            block.timestamp.to_rfc3339(),
            block.data,
            &block.hash[..16] // show first 16 chars only for brevity
        );
    }

    // Validate
    println!("\nValidating chain....");
    if chain.is_valid(){
        println!("Chain is valid!");
    }else{
        println!("Chain is Invalid!");
    }

    // Example tamper attempt
    println!("\nTampering with block 2's data to show validation:");
    //mutate (for demo) - in real the chain would be distributed, not mutable like this
    if chain.chain.len()>2{
        chain.chain[2].data="Bob pays Clara 5000NC (tampered)".to_string()
    }

    println!("Re-checking validity after tamper...");
    if chain.is_valid() {
        println!("✅ Chain is valid (unexpected)");
    } else {
        println!("❌ Chain is INVALID as expected after tampering");
    }
}