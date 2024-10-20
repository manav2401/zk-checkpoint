use alloy_rpc_types::BlockId;
use clap::Parser;
use sp1_sdk::{ProverClient, SP1Stdin};
use zk_checkpoint_lib::CheckpointProofInput;
use zk_checkpoint_lib::RootChainInfo;
use zk_checkpoint_lib::CALLER;
use zk_checkpoint_operator::utils::PoSClient;

use std::str::FromStr;
use url::Url;

use sp1_cc_client_executor::ContractInput;
use sp1_cc_host_executor::HostExecutor;

use alloy_primitives::Address;
use alloy_primitives::FixedBytes;
use alloy_provider::ReqwestProvider;
use alloy_rpc_types::BlockNumberOrTag;

pub const ELF: &[u8] = include_bytes!("../../../elf/riscv32im-succinct-zkvm-elf");

/// The arguments for the command.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    #[clap(long)]
    checkpoint_id: u64,

    #[clap(long)]
    checkpoint_tx_hash: String,

    #[clap(long)]
    l1_block_number: u64,

    #[arg(long, default_value_t = false)]
    prove: bool,
}

fn main() -> eyre::Result<()> {
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
    let proof = client
        .prove(&pk, stdin)
        .run()
        .expect("failed to generate proof");
    println!("Successfully generated proof!");

    // Verify the proof.
    client.verify(&proof, &vk).expect("failed to verify proof");
    println!("Successfully verified proof!");

    Ok(())
}

pub async fn generate_inputs(args: Args) -> eyre::Result<CheckpointProofInput> {
    let client = PoSClient::default();

    // Fetch checkpoint object and it's tx data
    let checkpoint = client.fetch_checkpoint_by_id(args.checkpoint_id).await?;
    let tx = client.fetch_tx_by_hash(args.checkpoint_tx_hash).await?;
    let tx_data = tx.result.tx;
    let tx_hash_str = tx.result.hash;
    let tx_hash = FixedBytes::from_str(&tx_hash_str).unwrap();

    // Fetch the block with precommits (i.e. n+2)
    let height: u64 = tx.result.height.parse().unwrap();
    let block = client.fetch_block_by_number(height + 2).await?;
    let block_precommits = block.result.block.last_commit.precommits;

    let mut sigs: Vec<String> = [].to_vec();
    let mut signers: Vec<Address> = [].to_vec();

    for precommit in block_precommits.iter() {
        // Only add if the side tx result is non empty
        if precommit.side_tx_results.is_some() {
            let side_tx = precommit.side_tx_results.as_ref().unwrap();
            for tx in side_tx.iter() {
                // Only add for requested checkpoint tx with success result
                if tx.tx_hash == tx_hash_str && tx.result == 1 {
                    sigs.push(tx.sig.clone().unwrap());
                    signers.push(Address::from_str(&precommit.validator_address).unwrap());
                }
            }
        }
    }

    let eth_rpc_url =
        std::env::var("ETH_RPC_URL").unwrap_or_else(|_| panic!("Missing ETH_RPC_URL in env"));
    let root_chain_info_address_str = std::env::var("ROOT_CHAIN_INFO")
        .unwrap_or_else(|_| panic!("Missing ROOT_CHAIN_INFO in env"));
    let root_chain_info_address = Address::from_str(&root_chain_info_address_str).unwrap();

    let block_number = BlockNumberOrTag::Number(args.l1_block_number);

    let provider = ReqwestProvider::new_http(Url::parse(&eth_rpc_url)?);
    let mut host_executor = HostExecutor::new(provider.clone(), block_number).await?;

    let l1_block_hash = host_executor.header.hash_slow();

    // Prepare and execute call to fetch last checkpoint from L1 contract
    let call = RootChainInfo::getLastCheckpointEndBlockCall {};
    let _response: RootChainInfo::getLastCheckpointEndBlockReturn = host_executor
        .execute(ContractInput {
            contract_address: root_chain_info_address,
            caller_address: CALLER,
            calldata: call,
        })
        .await?;

    // Prepare and execute call to fetch active validators from L1 contract
    let call = RootChainInfo::getActiveValidatorInfoCall {};
    let _response: RootChainInfo::getActiveValidatorInfoReturn = host_executor
        .execute(ContractInput {
            contract_address: root_chain_info_address,
            caller_address: CALLER,
            calldata: call,
        })
        .await?;

    // Assemble the evm sketch to be sent to prover
    let input = host_executor.finalize().await?;
    let state_sketch_bytes = bincode::serialize(&input)?;

    // Fetch the bor block corresponding to the end block of the checkpoint
    let bor_rpc_url =
        std::env::var("BOR_RPC_URL").unwrap_or_else(|_| panic!("Missing BOR_RPC_URL in env"));
    let provider = ReqwestProvider::new_http(Url::parse(&bor_rpc_url)?);
    let bor_block_number = BlockNumberOrTag::Number(checkpoint.result.end_block);
    let host_executor = HostExecutor::new(provider.clone(), bor_block_number).await?;
    let bor_block_hash = host_executor.header.hash_slow();

    Ok(CheckpointProofInput {
        tx_data,
        tx_hash,
        sigs,
        signers,
        state_sketch_bytes,
        root_chain_info_address,
        l1_block_hash,
        bor_block_hash,
    })
}
