// Write code here.
//
// To see what the code looks like after macro expansion:
//     $ cargo expand
//
// To run the code:
//     $ cargo run





fn main() {}

use bitfield::*;

const F: isize = 1;

#[derive(BitfieldSpecifier)]
pub enum DeliveryMode {
    Fixed = F,
    Lowest,
    SMI,
    RemoteRead,
    NMI,
    Init,
    Startup,
    External,
}
