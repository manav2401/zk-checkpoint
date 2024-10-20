use std::str::FromStr;

use alloy_primitives::{uint, FixedBytes, Uint};
use alloy_sol_types::SolType;
use sp1_sdk::{HashableKey, ProverClient};
use zk_checkpoint_lib::{CheckpointProofCommit, CommitStruct};

pub const ELF: &[u8] = include_bytes!("../../../elf/checkpoint-proof");

fn main() {
    sp1_sdk::utils::setup_logger();
    let client = ProverClient::new();
    let (_, vk) = client.setup(ELF);
    println!(
        "Program Verification Key: {}, Hash u32: {:?}",
        vk.bytes32(),
        vk.hash_u32()
    );
}
