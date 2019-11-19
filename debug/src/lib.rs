extern crate proc_macro;

use proc_macro::TokenStream;
use syn::spanned::Spanned;
use syn::{parse_macro_input, DeriveInput, Data, Fields, Ident, Attribute, Type, LitStr, Meta, Lit};
use quote::{quote, quote_spanned};
use proc_macro2::Span;

struct NamedStructField {
    ident: Ident,
    attr: Option<Attribute>, // at most 1
    ty: Type
}

struct NamedStructFieldWithAttr {
    ident: Ident,
    fmt: Option<LitStr>,
    ty: Type
}

fn gen_fmt_str(struct_name: &Ident, fields: &[NamedStructFieldWithAttr]) -> LitStr {
    let data = fields.iter().map(|f| {
        match &f.fmt {
            Some(l) => format!("{}: {}", f.ident, l.value()),
            None => format!("{}: {}", f.ident, "{:?}"),
        }
    }).collect::<Vec<_>>().join(", ");
    let ret = format!("{} {{{{ {} }}}}", struct_name, data);
    LitStr::new(&ret, Span::call_site())
}

fn extract_named_struct_fields_and_attr(input: &DeriveInput) -> Result<Vec<NamedStructFieldWithAttr>, TokenStream> {
    let fields = extract_named_struct_fields(input)?;
    let mut ret =  vec![];
    for field in fields {
        let lit = if let Some(attr) = &field.attr {
            Some(extract_format(attr)?)
        } else {
            None
        };
        ret.push(NamedStructFieldWithAttr {
            ident: field.ident,
            fmt: lit,
            ty: field.ty,
        })
    }
    Ok(ret)
}

fn extract_format(attr: &Attribute) -> Result<LitStr, TokenStream> {
    let meta = attr.parse_meta().map_err(|e|e.to_compile_error())?;
    let span = attr.span();
    if let Meta::NameValue(nv) = meta {
        if let Lit::Str(lstr) = nv.lit {
            return Ok(lstr)
        }
    }
    let e = quote_spanned! { span =>
        compile_error!("expected attribute with named value");
    };
    Err(e.into())
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
    let fields = extract_named_struct_fields_and_attr(input)?;
    let field_idents = fields.iter().map(|f| &f.ident);
    let struct_ident = &input.ident;
    let format_str = gen_fmt_str(&struct_ident, &fields);

    let ret = quote! {
        impl ::std::fmt::Debug for #struct_ident {
            fn fmt(&self, fmt: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                fmt.write_fmt(format_args!(#format_str  #(, self.#field_idents)*))
            }
        }
    };

    Ok(ret.into())
}

#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let ret = match derive_impl(&input)  {
        Ok(d) => d,
        Err(e) =>e,
    };

    ret.into()
}
