#![no_main]
sp1_zkvm::entrypoint!(main);

use alloy_primitives::{Address, Uint, B256};
use alloy_sol_types::SolType;
use checkpoint_proof::checkpoint::{prove, CheckpointProofInput, CommitStruct};

pub fn main() {
    let tx_data = sp1_zkvm::io::read::<String>();
    let tx_hash = sp1_zkvm::io::read::<B256>();
    let sigs = sp1_zkvm::io::read::<Vec<String>>();
    let signers = sp1_zkvm::io::read::<Vec<Address>>();
    let state_sketch_bytes = sp1_zkvm::io::read::<Vec<u8>>();
    let root_chain_info_address = sp1_zkvm::io::read::<Address>();
    let l1_block_hash = sp1_zkvm::io::read::<B256>();
    let bor_block_hash = sp1_zkvm::io::read::<B256>();
    let input = CheckpointProofInput {
        tx_data,
        tx_hash,
        sigs,
        signers,
        state_sketch_bytes,
        root_chain_info_address,
        l1_block_hash,
        bor_block_hash,
    };
    let commit = prove(input);

    let bytes = CommitStruct::abi_encode_packed(&CommitStruct {
        l1_block_hash: commit.l1_block_hash,
        bor_block_hash: commit.bor_block_hash,
        bor_block_number: Uint::from(commit.bor_block_number),
    });
    sp1_zkvm::io::commit_slice(&bytes);
}
