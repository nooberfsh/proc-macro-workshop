// Crates that have the "proc-macro" crate type are only allowed to export
// procedural macros. So we cannot have one crate that defines procedural macros
// alongside other types of public APIs like traits and structs.
//
// For this project we are going to need a #[bitfield] macro but also a trait
// and some structs. We solve this by defining the trait and structs in this
// crate, defining the attribute macro in a separate bitfield-impl crate, and
// then re-exporting the macro from this crate so that users only have one crate
// that they need to import.
//
// From the perspective of a user of this crate, they get all the necessary APIs
// (macro, trait, struct) through the one bitfield crate.
pub use bitfield_impl::bitfield;

// TODO other things
//

pub trait Specifier {
    const BITS: usize;
    const SIZE: usize = Self::BITS;
    type Container;
    
    //fn get(buf: &[u8]) -> Self::Container

    //fn set(buf: &[u8], data: Self::Container)
}


use seq::seq;

seq!(N in 1..=64 {
    pub enum B#N {}

    impl B#N {
        const fn size() -> usize  {
            let k = N / 8;
            let p = (N & 0b100 >> 2) | (N & 0b10 >> 1) | (N & 0b1);
            k + p
        }
    }
    
    impl Specifier for B#N {
        const BITS: usize = N;
        const SIZE: usize = B#N::size();
        type Container = u64;
    }
});
