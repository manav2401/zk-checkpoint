#![no_main]
sp1_zkvm::entrypoint!(main);

use alloy_primitives::Uint;
use alloy_sol_types::SolValue;
use zk_checkpoint_lib::{CheckpointProofInput, CommitStruct};
use zk_checkpoint_program::checkpoint::prove;

pub fn main() {
    let input = sp1_zkvm::io::read::<CheckpointProofInput>();
    let commit = prove(input);

    let bytes = CommitStruct::abi_encode_packed(&CommitStruct {
        l1_block_hash: commit.l1_block_hash,
        bor_block_hash: commit.bor_block_hash,
        bor_block_number: Uint::from_be_bytes(commit.bor_block_number.to_be_bytes()),
    });
    sp1_zkvm::io::commit_slice(&bytes);
}
