use alloy_sol_types::sol;
use alloy_primitives::{Address, B256};
use serde::{Serialize, Deserialize};

sol! {
    contract Verifier {
        function getLastCheckpointEndBlock() external view returns (uint256);
    }
}


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
    pub l1_block_hash: B256,
}

#[derive(Debug, Deserialize)]
pub struct CheckpointResponse {
    pub result: Checkpoint,
}

#[derive(Debug, Deserialize)]
pub struct Checkpoint {
    pub proposer: String,
    pub start_block: u64,
    pub end_block: u64,
    pub root_hash: String,
    pub bor_chain_id: String,
    pub timestamp: u64,
}

#[derive(Debug, Deserialize)]
pub struct TxResponse {
    pub result: TxResponseResult,
}

#[derive(Debug, Deserialize)]
pub struct TxResponseResult {
    pub hash: String,
    pub height: String,
    pub tx: String,
}

#[derive(Debug, Deserialize)]
pub struct BlockResponse {
    pub result: BlockResponseResult,
}

#[derive(Debug, Deserialize)]
pub struct BlockResponseResult {
    pub block: Block,
}

#[derive(Debug, Deserialize)]
pub struct Block {
    pub last_commit: LastCommit,
}

#[derive(Debug, Deserialize)]
pub struct LastCommit {
    pub precommits: Vec<Precommit>,
}

#[derive(Debug, Deserialize)]
pub struct Precommit {
    pub side_tx_results: Option<Vec<SideTxResult>>,
}

#[derive(Debug, Deserialize)]
pub struct SideTxResult {
    #[serde(rename = "tx_hash")]
    pub tx_hash: String,
    pub result: i32,
    pub sig: Option<String>,
}