#![feature(is_sorted)]

extern crate proc_macro;

use syn::*;
use syn::parse::{ParseStream, Parse};
use quote::*;
use proc_macro2::*;

struct SortedEnum {
    _attrs: Vec<Attribute>,
}

impl Parse for SortedEnum {
    fn parse(input: ParseStream) -> Result<SortedEnum> {
        Ok(SortedEnum{
            _attrs: input.call(Attribute::parse_outer)?,
        })
    }
}

#[proc_macro_attribute]
pub fn sorted(args: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let _attr = parse_macro_input!(args as SortedEnum);
    let input = parse_macro_input!(input as Item);
    let input = match extract_enum(&input) {
        Ok(d) => d,
        Err(e) => {
            let err = e.to_compile_error();
            let ret = quote!(#input #err);
            return ret.into();
        }
    };


    let ret = quote!(#input);
    ret.into()
}

fn extract_enum(input: &Item) -> Result<ItemEnum> {
    match input {
        Item::Enum(e) => {
            let _ = check_sorted(e)?;
            Ok(e.clone())
        },
        _ => Err(Error::new(Span::call_site(), "expected enum or match expression"))
    }
}

fn check_sorted(ie: &ItemEnum) -> Result<()> {
    let ret: Vec<_> =  ie.variants.clone().into_iter()
        .map(|v| v.ident)
        .collect();

    if ret.len() <= 1 { return Ok(()) }
    

    for i in 0..(ret.len() - 1) {
        if ret[i+1] < ret[i] {
            let bigger = find_bigger(&ret[0..=i], &ret[i+1]);
            return Err(Error::new(ret[i+1].span(), format!("{} should sort before {}", ret[i+1], bigger)));
        }
    }

    Ok(())
}

fn find_bigger<'a>(slice: &'a [Ident], id: &'a Ident) -> &'a Ident {
    assert!(slice.len() > 0);
    assert!(&slice[slice.len() - 1] > id);

    for i in (0..slice.len()).rev() {
        if &slice[i] > id && ( i == 0 || id >= &slice[i - 1] ) {
            return &slice[i]
        }
    }
    unreachable!()
}
