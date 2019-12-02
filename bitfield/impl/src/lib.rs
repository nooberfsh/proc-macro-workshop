extern crate proc_macro;

use syn::*;
use quote::*;
use syn::spanned::Spanned;
use proc_macro2::*;

#[proc_macro_attribute]
pub fn bitfield(args: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let _ = args;
    let _ = input;

    let s = parse_macro_input!(input as ItemStruct);

    match trans(s) {
        Ok(d) => d.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn trans(s: ItemStruct) -> Result<TokenStream> {
    let fields: Vec<Field> = match s.fields {
            Fields::Named(d) => {
                d.named.into_iter().collect()
            },
            Fields::Unnamed(d) => {
                return Err(Error::new(d.span(), "field can not be unit"))
            },
            Fields::Unit => return Err(Error::new(s.span(), "field can not be unit")),
    };

    let tyes: Vec<Type> = fields.iter().map(|f| f.ty.clone()).collect();
    let get_set: Vec<_> = fields.iter().map(|f| {
        let ty = &f.ty;
        let id = f.ident.as_ref().unwrap();

        let getter = format_ident!("get_{}", id);
        let setter = format_ident!("set_{}", id);

        quote! {
            fn #getter(&self) -> <#ty as Specifier>::Container {
                todo!()
            }

            fn #setter(&mut self, data: <#ty as Specifier>::Container) {
            }
        }
    }).collect();

    let name = &s.ident;

    let size = quote! {
        (#(<#tyes as Specifier>::BITS)+*) / 8
    };

    let ret = quote! {
        #[repr(C)]
        pub struct #name {
            data: [u8; #size],
        }

        impl #name {
            #(#get_set )*
        }
    };

    Ok(ret)
}
