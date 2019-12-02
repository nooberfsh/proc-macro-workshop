extern crate proc_macro;

use syn::*;
use quote::*;
use proc_macro2::*;
use syn::parse::{Parse, ParseStream};
use syn::token::Pound;

struct Bytes {
    ident: Ident,
    num: LitInt,
    n: u32
}

impl Parse for Bytes {
    fn parse(input: ParseStream) -> Result<Bytes> {
        let ident = input.parse()?;
        let _: Pound = input.parse()?;
        let num: LitInt = input.parse()?;

        let n = num.base10_parse::<u32>()?;

        if n <= 0 || n > 64 {
            return Err(Error::new(Span::call_site(), "n must > 0 and <= 64"));
        }
        
        Ok(Bytes{ident, num, n})
    }
}

#[proc_macro]
pub fn byte(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let bytes = parse_macro_input!(input as Bytes);

    let gen = (1..bytes.n).map(|i| {
        let name = format_ident!("{}{}", bytes.ident, i);
        
        let byte_size = byte_size(i, bytes.num.span());
        let bit_size = num_to_lit(i, bytes.num.span());
        let ty = i_to_ty(i);

        quote! {
            pub enum #name {}

            impl Specifier for #name {
                const BITS: usize = #bit_size;
                const SIZE: usize = #byte_size;
                type Container = #ty;
            }
        }
    });

    let ret = quote! {
        #(#gen )*
    };

    ret.into()
}

fn byte_size(i: u32, span: Span) -> LitInt  {
    assert!(i > 0);

    let mut k = i / 8;
    if i % 8 != 0 {
        k += 1;
    }
    
    num_to_lit(k, span)
}

fn num_to_lit(i: u32, span: Span) -> LitInt {
    LitInt::new(&format!("{}", i), span)
}

fn i_to_ty(bit_size: u32) -> Ident {
    assert!(bit_size > 0);

    let ret: u32 = if bit_size <= 8   {
        8
    } else if bit_size <= 16 {
        16
    } else if bit_size <= 32 {
        32
    } else if bit_size <= 64 {
        64
    } else {
        unreachable!()
    };
    format_ident!("u{}", ret)
}