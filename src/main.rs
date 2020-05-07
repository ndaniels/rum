// pub mod bitpack;
// pub mod memory;
// pub mod machine;
use rum::machine;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];
    let instructions = machine::boot(filename);
    machine::run(instructions);
}
