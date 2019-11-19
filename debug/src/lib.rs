extern crate proc_macro;

use proc_macro::TokenStream;
use syn::spanned::Spanned;
use syn::{parse_macro_input, DeriveInput, Data, Fields, Ident, Attribute, Type, LitStr};
use quote::{quote, quote_spanned};
use proc_macro2::Span;

struct NamedStructField {
    ident: Ident,
    attr: Option<Attribute>, // at most 1
    ty: Type
}

fn extract_named_struct_fields(input: &DeriveInput) -> Result<Vec<NamedStructField>, TokenStream> {
    let span = input.span();
    if let Data::Struct(s) = &input.data {
        if let Fields::Named(named) = &s.fields {
            let ret = named.named.iter().map(|f| {
                let attr = f.attrs.get(0).map(|a|a.clone());
                let ident = f.ident.clone().unwrap();
                let ty = f.ty.clone();
                NamedStructField{ident, attr, ty}
            }).collect();
            return Ok(ret)
        }
    }
    let e = quote_spanned! { span =>
        compile_error!("expected struct with named fields");
    };
    Err(e.into())
}

fn s<T: ToString>(t: &T) -> LitStr {
    LitStr::new(&format!("{}", t.to_string()), Span::call_site())
}

fn ss<T: ToString, I: IntoIterator<Item = T>>(i: I) -> impl Iterator<Item = LitStr> {
    i.into_iter().map(|d| s(&d))
}

fn derive_impl(input: &DeriveInput) -> Result<TokenStream, TokenStream> {
    let fields = extract_named_struct_fields(input)?;
    let field_idents = fields.iter().map(|f| &f.ident);
    let field_names = ss(field_idents.clone());
    let struct_ident = &input.ident;
    let struct_name = s(&struct_ident);

    let ret = quote! {
        impl ::std::fmt::Debug for #struct_ident {
            fn fmt(&self, fmt: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                fmt.debug_struct(#struct_name)
                    #(.field(#field_names, &self.#field_idents))*
                    .finish()
            }
        }
    };

    Ok(ret.into())
}

#[proc_macro_derive(CustomDebug)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let ret = match derive_impl(&input)  {
        Ok(d) => d,
        Err(e) =>e,
    };

    ret.into()
}
