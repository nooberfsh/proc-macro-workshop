// Write code here.
//
// To see what the code looks like after macro expansion:
//     $ cargo expand
//
// To run the code:
//     $ cargo run




use seq::seq;

seq!(N in 16..=20 {
    enum E {
        #(
            Variant#N,
        )*
    }
});


fn main() {
}


