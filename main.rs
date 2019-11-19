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
pub struct Field {
    name: &'static str,
    #[debug = "0b{:08b}"]
    bitmask: u8,
}



