extern crate crypto_hash;
extern crate serde_json;

use crypto_hash::{hex_digest, Algorithm};
//use std::io;



#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    pub id: String,
    pub timestamp: u64,
    pub payload: String,
}

//pub type Blockchain = Vec<Block>;




#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub index: u64,
    timestamp: u64,
    pub proof: u64,
    transactions: Vec<Transaction>,
    previous_block_hash: String,
}

pub const PREFIX: &str = "00";

impl Block {
    pub fn genesis() -> Self {
        let transaction = Transaction {
            id: String::from("1"),
            payload: String::from("This is dummy transaction as genesis block has no transactions"),
            timestamp: 0,
        };
        Block {
            index: 1,
            timestamp: 0,
            proof: 0,
            transactions: vec![transaction],
            previous_block_hash: String::from("0"),
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }

    pub fn hash(block: &Block) -> String {
        hex_digest(Algorithm::SHA256, block.to_json().as_bytes())
    }

    pub fn valid(hash: &str, prefix: &str) -> bool {
        hash.starts_with(prefix)
    }

    pub fn new(timestamp: u64, transactions: Vec<Transaction>, previous_block: &Block) -> Block {
        Block {
            index: previous_block.index + 1,
            timestamp: timestamp,
            proof: 0,
            transactions: transactions,
            previous_block_hash: Self::hash(previous_block),
        }
    }

    pub fn mine_without_iterator(block_candidate: &mut Block, prefix: &str) {
        while !Self::valid(&Self::hash(block_candidate), prefix) {
            println!("{}", block_candidate.proof);
            block_candidate.proof += 1
        }
    }
    pub fn _mine_with_iterator(block_candidate: &Block, prefix: &str) -> Block {
        (0..)
            .map(|proof| Block {
                index: block_candidate.index,
                timestamp: block_candidate.timestamp,
                proof: proof,
                transactions: block_candidate.transactions.clone(),
                previous_block_hash: block_candidate.previous_block_hash.clone(),
            })
            .find(|b| Self::valid(&Self::hash(b), prefix))
            .unwrap()
    }
}
/*
fn main() {
    println!("Hello, blockchain!");

    //create blockchain
    let mut blockchain: Blockchain = vec![Block::genesis()];
    println!("1: Blockchain is {:?}", blockchain);

    loop {
        let mut new_txn_text = String::new();
        println!(
            "So far {} blocks have been created. Please enter transaction details and press enter",
            blockchain.len()
        );
        io::stdin()
            .read_line(&mut new_txn_text)
            .expect("Failed to read transaction detail");
        let new_txn = Transaction {
            id: String::from("1"),
            timestamp: 0,
            payload: String::from(new_txn_text),
        };
        let mut new_block = Block::new(0, vec![new_txn], &blockchain[blockchain.len() - 1]);

        Block::mine_without_iterator(&mut new_block, &PREFIX);

        blockchain.push(new_block);
        for block in blockchain.iter() {
            println!("Block for index {} is {}", block.index, block.to_json());
        }
    }

}
*/