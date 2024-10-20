use std::collections::HashMap;

use crate::types::{self, *};

use base64::{prelude::BASE64_STANDARD, Engine};
use bincode;
use sha2::{Digest, Sha256};

use alloy_primitives::{keccak256, Address, FixedBytes, Uint, B256};
use alloy_sol_types::SolCall;
use reth_primitives::recover_signer_unchecked;
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

    // 3. Check if we have same number of sigs and signers
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

    // Construct a message which needs to be verified
    let mut message = vec![1]; // vote yes
    message.extend_from_slice(types::checkpoint_to_bytes(&checkpoint).as_slice());

    // 4. Verify the signatures of all validators
    for (i, sig) in input.sigs.iter().enumerate() {
        // check if it's a valid signer
        assert!(validator_stake_map.contains_key(&input.signers[i]));

        // verify
        verify_signature(sig.as_str(), &keccak256(message.clone()), input.signers[i]);

        // increase the majority power
        majority = majority.add_mod(validator_stake_map[&input.signers[i]], Uint::MAX);
    }

    // 5. Check if majority >= 2/3 of total stake
    let expected_majority = total_power
        .mul_mod(Uint::from(2), Uint::MAX)
        .div_ceil(Uint::from(3));
    if majority <= expected_majority {
        panic!("Majority voting power is less than 2/3rd of the total power, total_power: {}, majority_power: {}", total_power, majority);
    }

    CheckpointProofCommit {
        bor_block_hash: input.bor_block_hash,
        l1_block_hash: input.l1_block_hash,
        bor_block_number: checkpoint.end_block,
    }
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
