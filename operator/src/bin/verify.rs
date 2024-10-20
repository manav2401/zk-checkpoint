use sp1_sdk::{ProverClient, SP1ProofWithPublicValues};

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
