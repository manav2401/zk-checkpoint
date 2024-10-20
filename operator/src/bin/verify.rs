// use alloy_sol_types::{SolCall, SolType};
// use polccint_lib::pos::{ConsensusProofVerifier, PoSConsensusCommit, PublicValuesStruct};
// use pos_consensus_proof_host::{contract::ContractClient, ConsensusProver};
use sp1_sdk::{ProverClient, SP1ProofWithPublicValues};
use zk_checkpoint_operator::contract::ContractClient;
// use std::path::PathBuf;

pub const ELF: &[u8] = include_bytes!("../../../elf/riscv32im-succinct-zkvm-elf");

#[tokio::main]
async fn main() -> eyre::Result<()> {
    dotenv::dotenv().ok();

    println!("Loading proof...");
    let proof = SP1ProofWithPublicValues::load("proof.bin").expect("unable to load proof");

    let client = ProverClient::new();
    let (_, vk) = client.setup(ELF);

    println!("Verifying proof locally...");
    client.verify(&proof, &vk).expect("failed to verify proof");
    println!("Successfully verified proof!");

    Ok(())
}

// pub async fn verify_onchain(proof: SP1ProofWithPublicValues) -> eyre::Result<()> {
//     let contract_client = ContractClient::default();

//     // let commit = proof.public_values.

//     let consensus_commit = proof.public_values.clone().read::<PoSConsensusCommit>();

//     // Construct the on-chain call and relay the proof to the contract.
//     let call_data = ConsensusProofVerifier::verifyConsensusProofCall {
//         _proofBytes: proof.bytes().into(),
//         new_bor_block_hash: consensus_commit.new_bor_hash,
//         l1_block_hash: consensus_commit.l1_block_hash,
//     }
//     .abi_encode();
//     let result = contract_client.send(call_data).await;

//     if result.is_err() {
//         println!("error sending proof: err={:?}", result.err().unwrap());
//     } else {
//         println!("Successfully verified proof on-chain!");
//     }

//     Ok(())
// }
