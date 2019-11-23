// Write code here.
//
// To see what the code looks like after macro expansion:
//     $ cargo expand
//
// To run the code:
//     $ cargo run

fn main() {}


use derive_debug::CustomDebug;
use std::fmt::Debug;
use std::marker::PhantomData;

//pub trait Trait {
//    type Value;
//}
//
//#[derive(CustomDebug)]
//#[debug(bound = "T::Value: Debug")]
//pub struct Field<T: Trait, F, G> {
//    f: F,
//    g: PhantomData<G>,
//    bitmask: u8,
//    values: Vec<T::Value>,
//}




pub trait Trait {
    type Value;
}

#[derive(CustomDebug)]
#[debug(bound = "T::Value: Debug")]
pub struct Wrapper<T: Trait> {
    field: Field<T>,
}

#[derive(CustomDebug)]
struct Field<T: Trait> {
    values: Vec<T::Value>,
}
