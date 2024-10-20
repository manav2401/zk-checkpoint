use zk_checkpoint_lib::{CheckpointProofInput, CheckpointProofCommit};

pub fn prove(input: CheckpointProofInput) -> CheckpointProofCommit {    
    CheckpointProofCommit {
        l1_block_hash: input.l1_block_hash,
    }
}