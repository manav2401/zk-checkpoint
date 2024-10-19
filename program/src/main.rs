#![no_main]
sp1_zkvm::entrypoint!(main);

pub fn main() {
    let mut a = sp1_zkvm::io::read::<u64>();
    a+=1;
    sp1_zkvm::io::commit::<u64>(&a);
}
