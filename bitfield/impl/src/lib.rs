extern crate proc_macro;

//use syn::*;
//use quote::*;
use proc_macro2::*;

#[proc_macro_attribute]
pub fn bitfield(args: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let _ = args;
    let _ = input;

    TokenStream::new().into()
}
