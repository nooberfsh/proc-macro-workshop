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
pub use bitfield_impl::{bitfield, BitfieldSpecifier};
pub use byte::byte;

// TODO other things
//

pub trait Specifier {
    const BITS: usize;
    const SIZE: usize = Self::BITS;
    type Container;

    fn get(buf: &[u8], buf_idx: usize)  -> Self::Container;
    fn set(buf: &mut [u8], buf_idx: usize, data: Self::Container);
}

impl Specifier for bool {
    const BITS: usize = 1;
    const SIZE: usize = 1;
    type Container = bool;

    fn get(buf: &[u8], buf_idx: usize)  -> Self::Container {
        let byte = get_byte(buf, buf_idx, 1);
        byte != 0
    }
    fn set(buf: &mut [u8], buf_idx: usize, data: Self::Container) {
        let byte = data as u8;
        set_byte(buf, buf_idx, byte, 1);
    }
}

#[inline]
pub fn set_byte(buf: &mut [u8], buf_idx: usize, byte: u8, len: usize) {
    debug_assert!(len <= 8);

    let k = buf_idx % 8;
    let p = 8 - k;
    
    let head = buf[buf_idx / 8];

    if len <= p {
        let high = if len == p {
            0
        } else {
            let mask = (2u8.pow((len + k) as u32) - 1).reverse_bits();
            head & mask
        };

        let mid = byte << k;
        let mask = 2u8.pow(k as u32) - 1;
        let low = head & mask;
        buf[buf_idx / 8] = low | mid | high;
    } else {
        // handle first byte
        let mask_low = 2u8.pow(k as u32) - 1;
        let low = head & mask_low;
        let mask_high = 2u8.pow(p as u32) - 1;
        let high = (byte & mask_high) << k;
        buf[buf_idx / 8] = low & high;
        
        // handle next byte
        let low = byte >> p;
        let next = buf[buf_idx / 8 + 1];
        let mask_high = (2u8.pow((len - p) as u32) - 1).reverse_bits();
        let high =  next & mask_high;
        buf[buf_idx / 8 + 1] = low | high;
    }
}

#[inline]
pub fn get_byte(buf: &[u8], buf_idx: usize, len: usize)  -> u8 {
    debug_assert!(len <= 8);

    let k = buf_idx % 8;
    let p = 8 - k;

    let head = buf[buf_idx / 8];

    if len <=  p {
        if len == 8 {
            head
        } else {
            let mask =  2u8.pow(len as u32) - 1;
            ( head >> k ) & mask
        }
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

pub mod checks {
    pub trait TotalSizeIsMultipleOfEightBits {}
    pub trait Array { type Content; }

    pub struct ZeroMod8;
    pub struct SevenMod8;

    impl TotalSizeIsMultipleOfEightBits for ZeroMod8 {}
    
    impl Array  for [u8; 0] {
        type Content = ZeroMod8;
    }

    impl Array for [u8; 7] {
        type Content = SevenMod8;
    }

    /////////////////////////////////////////////////////
    pub trait DiscriminantInRange{}
    pub trait Array2 { type Content; }

    pub struct False;
    pub struct True;

    impl DiscriminantInRange for True {}
    
    impl Array2  for [u8; 0] {
        type Content = False;
    }

    impl Array2 for [u8; 1] {
        type Content = True;
    }
}
