use sp1_sdk::{ProverClient, SP1Stdin};

pub const ELF: &[u8] =
    include_bytes!("../../../elf/riscv32im-succinct-zkvm-elf");

fn main() {
    // Setup the logger.
    sp1_sdk::utils::setup_logger();

    // Setup the prover client.
    let client = ProverClient::new();

    // Setup the program for proving.
    let (pk, vk) = client.setup(ELF);
    
    // Setup the inputs.
    let mut stdin = SP1Stdin::new();
    stdin.write(&5);

    // Generate the proof.
    let proof = client.prove(&pk, stdin).run().expect("failed to generate proof");
    println!("Successfully generated proof!");

    // Verify the proof.
    client.verify(&proof, &vk).expect("failed to verify proof");
    println!("Successfully verified proof!");
}