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
    let get_set: Vec<_> = fields.iter().enumerate().map(|(i, f)| {
        let ty = &f.ty;
        let id = f.ident.as_ref().unwrap();

        let prev_tyes = &tyes[0..i];
        let idx_f_name = format_ident!("{}_idx", id);
        let idx_fn = if prev_tyes.is_empty() {
            quote! {
                fn #idx_f_name(&self) -> usize { 0 }
            }
        } else {
            quote! {
                fn #idx_f_name(&self) -> usize {
                    let prev_sum = #(<#prev_tyes as Specifier>::BITS)+*;
                    prev_sum
                }
            }
        };

        let getter = format_ident!("get_{}", id);
        let setter = format_ident!("set_{}", id);

        quote! {
            fn #setter(&mut self, data: <#ty as Specifier>::Container) {
                let buf_idx = self.#idx_f_name();
                let data = data.to_ne_bytes();
                let bit_size = <#ty as Specifier>::BITS;
                ::bitfield::set(&mut self.data, buf_idx, &data, bit_size);
            }

            fn #getter(&self) -> <#ty as Specifier>::Container {
                todo!()
            }

            #idx_fn
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
