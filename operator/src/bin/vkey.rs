use sp1_sdk::{HashableKey, ProverClient};

pub const ELF: &[u8] = include_bytes!("../../../elf/riscv32im-succinct-zkvm-elf");

fn main() {
    sp1_sdk::utils::setup_logger();
    let client = ProverClient::new();
    let (_, vk) = client.setup(ELF);
    println!(
        "Program Verification Key: {}, Hash u32: {:?}",
        vk.bytes32(),
        vk.hash_u32()
    );
}
