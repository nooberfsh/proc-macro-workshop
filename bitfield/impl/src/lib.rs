extern crate proc_macro;

use syn::*;
use quote::*;
use syn::spanned::Spanned;

#[proc_macro_attribute]
pub fn bitfield(args: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let _ = args;
    let _ = input;

    let s = parse_macro_input!(input as ItemStruct);

    match trans(s) {
        Ok(d) => {
            let ret = quote!(#d);
            ret.into()
        }
        Err(e) => e.to_compile_error().into(),
    }
}

fn trans(s: ItemStruct) -> Result<ItemStruct> {
    let tyes: Vec<Type> = match s.fields {
            Fields::Named(d) => {
                d.named.into_iter().map(|f| f.ty).collect()
            },
            Fields::Unnamed(d) => {
                d.unnamed.into_iter().map(|f| f.ty).collect()
            },
            Fields::Unit => return Err(Error::new(s.span(), "field can not be unit")),
    };

    let name = &s.ident;

    let size = quote! {
        (#(<#tyes as Specifier>::BITS)+*) / 8
    };

    let ret = parse_quote! {
        #[repr(C)]
        pub struct #name {
            data: [u8; #size],
        }
    };

    Ok(ret)
}
