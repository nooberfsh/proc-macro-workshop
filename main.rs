// Write code here.
//
// To see what the code looks like after macro expansion:
//     $ cargo expand
//
// To run the code:
//     $ cargo run

fn main() {}

use derive_builder::Builder2;

#[derive(Builder2)]
pub struct Command {
    executable: String,
    #[builder2(each = "arg")]
    args: Vec<String>,
    env: Vec<String>,
    current_dir: String,
}
