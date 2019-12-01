// Write code here.
//
// To see what the code looks like after macro expansion:
//     $ cargo expand
//
// To run the code:
//     $ cargo run




trait Specifier {
    const BITS: usize;
}


use seq::seq;

seq!(N in 1..=64 {
    enum B#N {}
    
    impl Specifier for B#N {
        const BITS: usize = N;
    }
});

fn main() {}
