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
pub use byte::byte;

// TODO other things
//

pub trait Specifier {
    const BITS: usize;
    const SIZE: usize = Self::BITS;
    type Container;
}

pub fn set(buf: &mut [u8], buf_idx: usize, data: &[u8], bit_size: usize) {
    todo!()
}

#[inline]
pub fn get_byte(buf: &[u8], buf_idx: usize, len: usize)  -> u8 {
    debug_assert!(len <= 8);

    let k = buf_idx % 8;
    let p = 8 - k;

    let head = buf[buf_idx / 8];

    if len <=  p {
        let mask =  2u8.pow(len as u32) - 1;
        ( head >> k ) & mask
    } else {
        let next = buf[buf_idx / 8 + 1];
        let left = len - p;
        let mask = 2u8.pow(left as u32) - 1;
        let high =  ( next & mask ) << p;
        let low = head >> k;
        low & high
    }
}


byte!(B#64);
