use crate::{
    checkpoint::{RootChainInfo, CALLER},
    types::*,
};

use base64::{prelude::BASE64_STANDARD, Engine};
use core::str;
use sha2::{Digest, Sha256};

use alloy_primitives::{Address, FixedBytes, Uint, B256};
use alloy_sol_types::{sol, SolCall};
use reth_primitives::recover_signer_unchecked;
use sp1_cc_client_executor::{io::EVMStateSketch, ClientExecutor, ContractInput};

fn sha256(decoded_tx_data: &[u8]) -> FixedBytes<32> {
    // Create a new Sha256 instance
    let mut hasher = Sha256::new();

    // Write the tx data
    hasher.update(decoded_tx_data);

    // Read hash digest and consume hasher
    let result = hasher.finalize();

    FixedBytes::from_slice(result.as_slice())
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

pub fn verify_signature(signature: &str, message_hash: &[u8; 32], expected_signer: Address) {
    let decoded_signature = BASE64_STANDARD
        .decode(signature)
        .expect("unable to decode signature");

    // Construct the byte array from the decoded signature for recovery
    let mut sig = [0u8; 65];
    sig.copy_from_slice(decoded_signature.as_slice());

    let recovered_signer = recover_signer_unchecked(&sig, message_hash).unwrap_or_default();
    let recovered_signer_alloy = Address::from_slice(recovered_signer.as_slice());

    assert_eq!(
        expected_signer, recovered_signer_alloy,
        "recovered and expected signature mismatch"
    );
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
