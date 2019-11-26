// Write code here.
//
// To see what the code looks like after macro expansion:
//     $ cargo expand
//
// To run the code:
//     $ cargo run




use eseq::eseq;

fn main() {
    let tuple = (9u8, 90u16, 900u32, 9000u64);

    let mut sum = 0;

    eseq!(N in 0..4 {{
        #(
            sum += tuple.N as u64;
        )*
    }});

    assert_eq!(sum, 9999);
}


