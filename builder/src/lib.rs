extern crate proc_macro;

use std::collections::HashMap;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{
    parse_macro_input, parse_quote, Data, DeriveInput, Field, Fields, GenericArgument, Ident, Lit,
    LitStr, Meta, NestedMeta, PathArguments, Type,
};

fn is_option(ty: &Type) -> bool {
    if let Type::Path(ref p) = ty {
        let path = &p.path;
        path.leading_colon.is_none()
            && path.segments.len() == 1
            && path.segments.iter().next().unwrap().ident == "Option"
    } else {
        false
    }
}

fn wrap_option_ty(ty: Type) -> Type {
    if is_option(&ty) {
        ty
    } else {
        parse_quote! {
            Option<#ty>
        }
    }
}

fn extract_ty_from_option(ty: Type) -> Type {
    if !is_option(&ty) {
        return ty;
    }

    if let Type::Path(tp) = ty {
        let seg = tp.path.segments.into_iter().next().unwrap();
        if let PathArguments::AngleBracketed(args) = seg.arguments {
            let generic_arg = args.args.into_iter().next().unwrap();
            if let GenericArgument::Type(new_ty) = generic_arg {
                return new_ty;
            }
        }
    }
    unreachable!()
}

fn wrap_option_field(mut f: Field) -> Field {
    let ty = wrap_option_ty(f.ty);
    f.ty = ty;
    f.attrs = vec![];
    f
}

fn check_builder_attribute(f: &Field) -> Result<Option<String>, TokenStream> {
    if f.attrs.is_empty() {
        return Ok(None);
    }

    assert!(f.attrs.len() == 1);

    let attr = &f.attrs[0];
    //assert!(format!("{:?}", attr.path) == "builder");

    let meta = attr.parse_meta().unwrap();
    if let Meta::List(l) = meta {
        if l.nested.len() == 1 {
            if let NestedMeta::Meta(Meta::NameValue(nv)) = l.nested.into_iter().next().unwrap() {
                if let Lit::Str(lstr) = nv.lit {
                    if let Some(ident) = nv.path.get_ident() {
                        if ident == "each" {
                            let val = lstr.value();
                            return Ok(Some(val));
                        }
                    }
                }
            }
        }
    }
    let ret = quote_spanned! { attr.span() =>
        compile_error!(r#"expected `builder2(each = "...")`"#);
    };
    Err(ret.into())
}

#[proc_macro_derive(Builder2, attributes(builder2))]
pub fn derive2(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_name = input.ident;
    let builder_name = Ident::new(&format!("{}Builder", struct_name), Span::call_site());

    let fields = if let Data::Struct(s) = input.data {
        if let Fields::Named(named) = s.fields {
            named.named
        } else {
            panic!("only used to named fields")
        }
    } else {
        panic!("only used to struct")
    };

    let mut ty2attr: HashMap<Ident, String> = HashMap::new();

    for f in fields.clone() {
        match check_builder_attribute(&f) {
            Ok(Some(d)) => {
                ty2attr.insert(f.ident.clone().unwrap(), d);
            }
            Ok(None) => {}
            Err(e) => return e,
        };
    }

    let option_fields = fields.clone().into_iter().map(wrap_option_field);

    let ret = quote! {
        pub struct #builder_name {
            #(#option_fields, )*
        }
    };

    ret.into()
}

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_name = input.ident;
    let builder_name = Ident::new(&format!("{}Builder", struct_name), Span::call_site());

    let fields = if let Data::Struct(s) = input.data {
        if let Fields::Named(named) = s.fields {
            named.named
        } else {
            panic!("only used to named fields")
        }
    } else {
        panic!("only used to struct")
    };

    let option_fields = fields.clone().into_iter().map(wrap_option_field);

    let setters = fields.clone().into_iter().map(|f| {
        let name = f.ident.clone().unwrap();
        let raw_ty = extract_ty_from_option(f.ty.clone());
        quote! {
            pub fn #name(&mut self, #name: #raw_ty) -> &mut Self {
                self.#name = Some(#name);
                self
            }
        }
    });

    let constructor = fields.clone().into_iter().map(|f| {
        let name = f.ident.unwrap();
        let err = LitStr::new(&format!("field {} not set", name), Span::call_site());
        if is_option(&f.ty) {
            quote! {
                #name: self.#name.take()
            }
        } else {
            quote! {
                #name: self.#name.take().ok_or(#err)?
            }
        }
    });

    let ret = quote! {
        impl #struct_name {
            pub fn builder() -> #builder_name {
                Default::default()
            }
        }

        #[derive(Default)]
        pub struct #builder_name {
            #(#option_fields, )*
        }

        impl #builder_name {
           #(#setters )*

            pub fn build(&mut self) -> Result<#struct_name, Box<dyn std::error::Error>> {
                Ok(#struct_name {
                    #(#constructor, )*
                })
            }
        }
    };

    ret.into()
}
