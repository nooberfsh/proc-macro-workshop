extern crate proc_macro;

use syn::spanned::Spanned;
use syn::*;
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

fn generic_arg_contain_ident(ga: &GenericArgument, ident: &Ident) -> Option<Kind> {
    if let GenericArgument::Type(ty) = ga {
        ty_contain_ident(ty, ident)
    } else {
        None
    }
}

fn path_args_contain_ident(pa: &PathArguments, ident: &Ident) -> Option<Kind> {
    match pa {
        PathArguments::AngleBracketed(args) => {
            args
                .args
                .iter()
                .find_map(|arg| generic_arg_contain_ident(arg, ident))
        },
        _ => None,
    }
}

fn ty_contain_ident(ty: &Type, ident: &Ident) -> Option<Kind> {
    match ty {
        Type::Path(ty) if is_phantom(ty) => None,
        Type::Path(ty) => {
            let segs = &ty.path.segments;
            let first =  segs.first().unwrap();
            if ty.path.leading_colon.is_none()  && &first.ident == ident {
                if segs.len() == 1 {
                    Some(Kind::NotPath)
                } else {
                    Some(Kind::Path(ty.path.clone()))
                }
            } else {
                let last = segs.last().unwrap();
                path_args_contain_ident(&last.arguments, ident)
            }
        },
        Type::Tuple(ty) =>  {
            ty
                .elems
                .iter()
                .find_map(|ty| ty_contain_ident(ty, ident))
        },
        Type::Array(ty) => ty_contain_ident(&ty.elem, ident),
        Type::Group(ty) => ty_contain_ident(&ty.elem, ident),
        Type::Paren(ty) => ty_contain_ident(&ty.elem, ident),
        Type::Ptr(ty) => ty_contain_ident(&ty.elem, ident),
        Type::Reference(ty) => ty_contain_ident(&ty.elem, ident),
        Type::Slice(ty) => ty_contain_ident(&ty.elem, ident),
        Type::ImplTrait(_ty) => unreachable!(),
        Type::Infer(_ty) => unreachable!(),
        Type::Macro(_ty) => unreachable!(),
        Type::BareFn(_ty) =>  None,
        Type::Never(_ty) => None,
        Type::TraitObject(_ty) => None,
        Type::Verbatim(_ty) => None,
        _ => panic!("unkown type"),
    }
}

enum Kind {
    NotPath,
    Path(Path),
}

fn is_phantom(tp: &TypePath) -> bool {
    if let Some(ps) = tp.path.segments.last() {
        ps.ident == "PhantomData"
    } else {
        false
    }
}

fn extract_named_struct_fields_and_attr(input: &DeriveInput) -> std::result::Result<Vec<NamedStructFieldWithAttr>, proc_macro::TokenStream> {
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

fn extract_format(attr: &Attribute) -> std::result::Result<LitStr, proc_macro::TokenStream> {
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

fn extract_named_struct_fields(input: &DeriveInput) -> std::result::Result<Vec<NamedStructField>, proc_macro::TokenStream> {
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

// add debug bound
fn add_trait_bounds(mut generics: Generics, fields: &[NamedStructFieldWithAttr]) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            match need_bound(type_param, fields) {
                Some(Kind::NotPath) =>
                    type_param.bounds.push(parse_quote!(::std::fmt::Debug)),
                Some(Kind::Path(p)) => {
                    match generics.where_clause {
                        Some(ref mut w) => {
                            let cond = parse_quote!(#p: ::std::fmt::Debug);
                            w.predicates.push_value(cond);
                        },
                        None => {
                            let w = parse_quote!(where #p: ::std::fmt::Debug);
                            generics.where_clause = Some(w);
                        }
                    }
                }
                None => {},
            }
        }
    }
    generics
}

fn need_bound(tp: &TypeParam, fields: &[NamedStructFieldWithAttr]) -> Option<Kind> {
    fields.iter().find_map(|field| ty_contain_ident(&field.ty, &tp.ident))
}

fn derive_impl(input: &DeriveInput) -> std::result::Result<proc_macro::TokenStream, proc_macro::TokenStream> {
    let fields = extract_named_struct_fields_and_attr(input)?;
    let field_idents = fields.iter().map(|f| &f.ident);
    let struct_ident = &input.ident;
    let format_str = gen_fmt_str(&struct_ident, &fields);

    let generics = add_trait_bounds(input.generics.clone(), &fields);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let ret = quote! {
        impl #impl_generics ::std::fmt::Debug for #struct_ident #ty_generics #where_clause {
            fn fmt(&self, fmt: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                fmt.write_fmt(format_args!(#format_str  #(, self.#field_idents)*))
            }
        }
    };

    Ok(ret.into())
}

#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let ret = match derive_impl(&input)  {
        Ok(d) => d,
        Err(e) =>e,
    };

    ret.into()
}
