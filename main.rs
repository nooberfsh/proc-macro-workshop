// Write code here.
//
// To see what the code looks like after macro expansion:
//     $ cargo expand
//
// To run the code:
//     $ cargo run

fn main() {}

use derive_debug::CustomDebug;

#[derive(CustomDebug)]
pub struct XxField {
    name: &'static str,
    bitmask: u8,
}


