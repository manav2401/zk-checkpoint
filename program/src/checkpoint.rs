use alloy_primitives::{Address, B256};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointProofInput {
    pub tx_data: String,
    pub tx_hash: String,
    pub sigs: Vec<String>,
    pub signers: Vec<Address>,
    pub state_sketch_bytes: Vec<u8>,
    pub l1_block_hash: B256,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointProofCommit {
    l1_block_hash: B256,
}

pub fn prove(input: CheckpointProofInput) -> CheckpointProofCommit {    
    CheckpointProofCommit {
        l1_block_hash: input.l1_block_hash,
    }
}