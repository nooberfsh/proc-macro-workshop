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

pub trait Trait {
    type Value;
}

#[derive(CustomDebug)]
pub struct Field<T: Trait, F, G> {
    f: F,
    g: PhantomData<G>,
    values: Vec<T::Value>,
}

