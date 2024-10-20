use alloy_primitives::{address, Address, B256};
use alloy_sol_types::sol;
use serde::{Deserialize, Serialize};

pub const CALLER: Address = address!("0000000000000000000000000000000000000000");

sol! {
    contract RootChainInfo {
        function getLastCheckpointEndBlock() external view returns (uint256);
        function getActiveValidatorInfo() public view returns(address[] memory, uint256[] memory, uint256);
    }
}

sol! {
    struct CommitStruct {
        bytes32 l1_block_hash;
        bytes32 bor_block_hash;
        uint256 bor_block_number;
    }
}

sol! {
    contract PoSVerifier {
        function verifyCheckpointSignatures(
            bytes calldata _proofBytes,
            bytes32 _l1BlockHash,
            bytes32 _borBlockHash,
            uint256 _borBlockNumber
        ) public;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointProofInput {
    pub tx_data: String,
    pub tx_hash: B256,
    pub sigs: Vec<String>,
    pub signers: Vec<Address>,
    pub state_sketch_bytes: Vec<u8>,
    pub root_chain_info_address: Address,
    pub l1_block_hash: B256,
    pub bor_block_hash: B256,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointProofCommit {
    pub l1_block_hash: B256,
    pub bor_block_hash: B256,
    pub bor_block_number: u64,
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
