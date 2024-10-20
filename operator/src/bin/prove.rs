use sp1_sdk::{ProverClient, SP1Stdin};
use zk_checkpoint_operator::utils::PoSClient;
use zk_checkpoint_lib::CheckpointProofInput;
use clap::Parser;

pub const ELF: &[u8] =
    include_bytes!("../../../elf/riscv32im-succinct-zkvm-elf");

/// The arguments for the command.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    #[clap(long)]
    checkpoint_id: u64,

    #[clap(long)]
    checkpoint_tx_hash: String,

    #[arg(long, default_value_t = false)]
    prove: bool,
}

fn main() -> eyre::Result<()>{
    dotenv::dotenv().ok();

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

    Ok(())
}

pub async fn generate_inputs(args: Args) -> eyre::Result<()>{
    let client = PoSClient::default();

    // Fetch checkpoint object and it's tx data
    let checkpoint = client.fetch_checkpoint_by_id(args.checkpoint_id).await?;
    let tx = client.fetch_tx_by_hash(args.checkpoint_tx_hash).await?;

    // Fetch the block with precommits (i.e. n+2)
    let height: u64 = tx.result.height.parse().unwrap();
    let block = client.fetch_block_by_number(height+2).await?;

    Ok(())
}