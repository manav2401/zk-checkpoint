// use alloy_sol_types::{SolCall, SolType};
// use polccint_lib::pos::{ConsensusProofVerifier, PoSConsensusCommit, PublicValuesStruct};
// use pos_consensus_proof_host::{contract::ContractClient, ConsensusProver};
use alloy_sol_types::{SolCall, SolType};
use sp1_sdk::{ProverClient, SP1ProofWithPublicValues};
use zk_checkpoint_lib::{CommitStruct, PoSVerifier};
use zk_checkpoint_operator::contract::ContractClient;

pub const ELF: &[u8] = include_bytes!("../../../elf/checkpoint-proof");

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

    println!("Verifying proof on-chain...");
    verify_onchain(proof).await?;
    println!("Successfully verified proof on-chain!");

    Ok(())
}

pub async fn verify_onchain(proof: SP1ProofWithPublicValues) -> eyre::Result<()> {
    let contract_client = ContractClient::default();

    // Decode the commit from proof
    let commit = CommitStruct::abi_decode(&proof.public_values.to_vec(), true).unwrap();

    // Call verifier contract
    let call_data = PoSVerifier::verifyCheckpointSignaturesCall {
        _proofBytes: proof.bytes().into(),
        _l1BlockHash: commit.l1_block_hash,
        _borBlockHash: commit.bor_block_hash,
        _borBlockNumber: commit.bor_block_number,
    }
    .abi_encode();
    let result = contract_client.send(call_data).await;

    if result.is_err() {
        println!("error sending proof: err={:?}", result.err().unwrap());
    } else {
        println!("Successfully verified proof on-chain!");
    }

    Ok(())
}
