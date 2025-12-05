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
        
    }
}