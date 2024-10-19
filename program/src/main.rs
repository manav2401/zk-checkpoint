#![no_main]

use zk_checkpoint_program::checkpoint::{prove, CheckpointProofCommit, CheckpointProofInput};
sp1_zkvm::entrypoint!(main);

pub fn main() {
    let input = sp1_zkvm::io::read::<CheckpointProofInput>();
    let commit = prove(input);
    sp1_zkvm::io::commit::<CheckpointProofCommit>(&commit);
}
