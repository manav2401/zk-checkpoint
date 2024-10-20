use std::collections::HashMap;

use crate::types::*;

use base64::{prelude::BASE64_STANDARD, Engine};
use bincode;
use sha2::{Digest, Sha256};

use alloy_primitives::{Address, FixedBytes, Uint, B256};
use alloy_sol_types::SolCall;
use sp1_cc_client_executor::{io::EVMStateSketch, ClientExecutor, ContractInput};
use zk_checkpoint_lib::{CheckpointProofCommit, CheckpointProofInput, RootChainInfo, CALLER};

pub fn prove(input: CheckpointProofInput) -> CheckpointProofCommit {
    // 1. validate tx: hash(tx_data) == tx_hash
    let checkpoint = validate_checkpoint_msg(&input.tx_data, &input.tx_hash);

    // 2. checkpoint.start_block = last_checkpoint_end_block + 1
    validate_checkpoint(
        checkpoint.start_block,
        input.root_chain_info_address,
        input.state_sketch_bytes.clone(),
    );

    // 3. Check if we have same number of side_txs, sigs and signers
    assert_eq!(input.side_txs.len(), input.sigs.len());
    assert_eq!(input.sigs.len(), input.signers.len());

    // Fetch the validator info (addresses and stake) from the PoS contracts
    let (signers, powers, total_power) = fetch_validator_info(
        input.root_chain_info_address,
        input.state_sketch_bytes.clone(),
    );

    // Combine the data into a map of address -> total stake
    let mut validator_stake_map = HashMap::new();
    for (i, signer) in signers.iter().enumerate() {
        validator_stake_map.insert(signer, powers[i]);
    }

    // Initialise the majority stake
    let mut majority: Uint<256, 4> = Uint::from(0);

    CheckpointProofCommit {
        l1_block_hash: input.l1_block_hash,
    }
}

pub fn fetch_validator_info(
    root_chain_info_address: Address,
    state_sketch_bytes: Vec<u8>,
) -> (Vec<Address>, Vec<Uint<256, 4>>, Uint<256, 4>) {
    let state_sketch = bincode::deserialize::<EVMStateSketch>(&state_sketch_bytes).unwrap();
    let executor = ClientExecutor::new(state_sketch).unwrap();

    // Call `getActiveValidatorInfo` on respective L1
    let call = RootChainInfo::getActiveValidatorInfoCall {};
    let call_input = ContractInput {
        contract_address: root_chain_info_address,
        caller_address: CALLER,
        calldata: call.clone(),
    };
    let output = executor.execute(call_input).unwrap();
    let response =
        RootChainInfo::getActiveValidatorInfoCall::abi_decode_returns(&output.contractOutput, true)
            .unwrap();

    (response._0, response._1, response._2)
}

pub fn validate_checkpoint(
    start_block: u64,
    root_chain_info_address: Address,
    state_sketch_bytes: Vec<u8>,
) {
    let state_sketch = bincode::deserialize::<EVMStateSketch>(&state_sketch_bytes).unwrap();
    let executor = ClientExecutor::new(state_sketch).unwrap();

    // Call `getLastCheckpointEndBlock` on respective L1
    let call = RootChainInfo::getLastCheckpointEndBlockCall {};
    let call_input = ContractInput {
        contract_address: root_chain_info_address,
        caller_address: CALLER,
        calldata: call.clone(),
    };
    let output = executor.execute(call_input).unwrap();
    let response = RootChainInfo::getLastCheckpointEndBlockCall::abi_decode_returns(
        &output.contractOutput,
        true,
    )
    .unwrap();

    let last_end = response._0;
    let last_end_u64: u64 = last_end
        .try_into()
        .expect("failed to convert last_end value to u64");
    assert_eq!(start_block, last_end_u64 + 1);
}

pub fn validate_checkpoint_msg(
    tx_data: &str,
    expected_hash: &B256,
) -> heimdall_types::CheckpointMsg {
    // Decode the checkpoint tx
    let mut decoded_tx_data = BASE64_STANDARD
        .decode(tx_data)
        .expect("failed to decode checkpoint tx data");
    let tx_hash = sha256(decoded_tx_data.as_slice());

    assert_eq!(*expected_hash, tx_hash);

    let checkpoint_msg = deserialize_checkpoint_tx(&mut decoded_tx_data)
        .expect("failed to deserialize checkpoint tx data");

    checkpoint_msg.msg.unwrap()
}

fn sha256(decoded_tx_data: &[u8]) -> FixedBytes<32> {
    let mut hasher = Sha256::new();
    hasher.update(decoded_tx_data);
    let result = hasher.finalize();
    FixedBytes::from_slice(result.as_slice())
}
