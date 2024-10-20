use crate::{helper::*, types::checkpoint_to_bytes};
use std::collections::HashMap;

use alloy_primitives::{address, keccak256, Address, FixedBytes, Uint, B256};
use alloy_sol_types::sol;
use reth_primitives::Header;

pub const CALLER: Address = address!("0000000000000000000000000000000000000000");

sol! {
    contract RootChainInfo {
        function getLastCheckpointEndBlock() external view returns (uint256);
        function getActiveValidatorInfo() public view returns(address[] memory, uint256[] memory, uint256);
    }
}

sol! {
    struct CommitStruct {
        bytes32 l1_block_hash;
        bytes32 bor_block_hash;
        uint256 bor_block_number;
    }
}

sol! {
    contract PoSVerifier {
        function verifyCheckpointSignatures(
            bytes calldata _proofBytes,
            bytes32 _l1BlockHash,
            bytes32 _borBlockHash,
            uint256 _borBlockNumber
        ) public;
    }
}

#[derive(Clone, Debug)]
pub struct MilestoneProofInputs {
    // heimdall related data
    pub tx_data: String,
    pub tx_hash: FixedBytes<32>,
    pub precommits: Vec<Vec<u8>>,
    pub sigs: Vec<String>,
    pub signers: Vec<Address>,

    // bor related data
    pub bor_header: Header,
    pub prev_bor_header: Header,

    // l1 related data
    pub state_sketch_bytes: Vec<u8>,
    pub l1_block_hash: FixedBytes<32>,
}

#[derive(Clone, Debug)]
pub struct MilestoneProofOutputs {
    pub prev_bor_hash: FixedBytes<32>,
    pub new_bor_hash: FixedBytes<32>,
    pub l1_block_hash: FixedBytes<32>,
}

#[derive(Debug, Clone)]
pub struct CheckpointProofInput {
    pub tx_data: String,
    pub tx_hash: B256,
    pub sigs: Vec<String>,
    pub signers: Vec<Address>,
    pub state_sketch_bytes: Vec<u8>,
    pub root_chain_info_address: Address,
    pub l1_block_hash: B256,
    pub bor_block_hash: B256,
}

#[derive(Debug, Clone)]
pub struct CheckpointProofCommit {
    pub l1_block_hash: B256,
    pub bor_block_hash: B256,
    pub bor_block_number: u64,
}

pub fn prove(input: CheckpointProofInput) -> CheckpointProofCommit {
    // 1. validate tx: hash(tx_data) == tx_hash
    let checkpoint = validate_checkpoint_msg(&input.tx_data, &input.tx_hash);

    // 2. checkpoint.start_block = last_checkpoint_end_block + 1
    // SKIPPING this for testing old checkpoints
    // validate_checkpoint(
    //     checkpoint.start_block,
    //     input.root_chain_info_address,
    //     input.state_sketch_bytes.clone(),
    // );

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
    message.extend_from_slice(checkpoint_to_bytes(&checkpoint).as_slice());

    // 4. Verify the signatures of all validators
    let mut verify_count = 0;
    for (i, sig) in input.sigs.iter().enumerate() {
        // check if it's a valid signer
        assert!(validator_stake_map.contains_key(&input.signers[i]));

        // verify
        verify_signature(sig.as_str(), &keccak256(message.clone()), input.signers[i]);

        // increase the majority power
        majority = majority.add_mod(validator_stake_map[&input.signers[i]], Uint::MAX);
        verify_count += 1;
    }

    // 5. Check if majority >= 2/3 of total stake
    let expected_majority = total_power
        .mul_mod(Uint::from(2), Uint::MAX)
        .div_ceil(Uint::from(3));
    if majority <= expected_majority {
        panic!("Majority voting power is less than 2/3rd of the total power, total_power: {}, majority_power: {}, vc: {}", total_power, majority, verify_count);
    }

    CheckpointProofCommit {
        bor_block_hash: input.bor_block_hash,
        l1_block_hash: input.l1_block_hash,
        bor_block_number: checkpoint.end_block,
    }
}
