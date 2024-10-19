use sp1_helper::{build_program_with_args, BuildArgs};

fn main() {
    let args = BuildArgs {
        ignore_rust_version: true,
        ..Default::default()
    };
    build_program_with_args("../program", args);
}
