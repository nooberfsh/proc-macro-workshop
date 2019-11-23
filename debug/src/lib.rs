extern crate proc_macro;

use proc_macro::TokenStream;
use syn::spanned::Spanned;
use syn::{parse_macro_input, parse_quote, DeriveInput, Data, Fields, Ident, Attribute, Type, LitStr, Meta, Lit, Generics, GenericParam, TypeParam, PathSegment, GenericArgument};
use syn::PathArguments;
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

fn generic_arg_contain_ident(ga: &GenericArgument, ident: &Ident) -> bool {
    if let GenericArgument::Type(ty) = ga {
        ty_contain_ident(ty, ident)
    } else {
        false
    }
}

fn path_seg_contain_ident(ps: &PathSegment, ident: &Ident) -> bool {
    if &ps.ident == ident {
        true
    } else if let PathArguments::AngleBracketed(arg) = &ps.arguments {
        arg.args.iter().find(|arg| generic_arg_contain_ident(arg, ident)).is_some()
    } else {
        false
    }
}

fn ty_contain_ident(ty: &Type, ident: &Ident) -> bool {
    match ty {
        Type::Path(ty) => {
            let seg = ty.path.segments.iter().last().unwrap();
            path_seg_contain_ident(seg, ident)
        },
        Type::Array(ty) => ty_contain_ident(&ty.elem, ident),
        Type::BareFn(_ty) =>  false,
        Type::Group(ty) => ty_contain_ident(&ty.elem, ident),
        Type::ImplTrait(_ty) => unreachable!(),
        Type::Infer(_ty) => unreachable!(),
        Type::Macro(_ty) => unreachable!(),
        Type::Never(_ty) => false,
        Type::Paren(ty) => ty_contain_ident(&ty.elem, ident),
        Type::Ptr(ty) => ty_contain_ident(&ty.elem, ident),
        Type::Reference(ty) => ty_contain_ident(&ty.elem, ident),
        Type::Slice(ty) => ty_contain_ident(&ty.elem, ident),
        Type::TraitObject(_ty) => false,
        Type::Tuple(ty) => ty.elems.iter().find(|ty| ty_contain_ident(ty, ident)).is_some(),
        Type::Verbatim(_ty) => false,
        _ => false,
    }
}

fn is_phantom(ty: &Type) -> bool {
    match ty {
        Type::Path(ty) => {
            let p = &ty.path.segments.iter().last().unwrap().ident;
            p == "PhantomData"
        },
//        Type::Array(ty) => todo!(),
//        Type::BareFn(ty) =>  todo!(),
//        Type::Group(ty) => todo!(),
//        Type::ImplTrait(ty) => todo!(),
//        Type::Infer(ty) => todo!(),
//        Type::Macro(ty) => todo!(),
//        Type::Never(ty) => todo!(),
//        Type::Paren(ty) => todo!(),
//        Type::Ptr(ty) => todo!(),
//        Type::Reference(ty) => todo!(),
//        Type::Slice(ty) => todo!(),
//        Type::TraitObject(ty) => todo!(),
//        Type::Tuple(ty) => todo!(),
//        Type::Verbatim(ty) => todo!(),
        _ => false
    }
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

// add debug bound
fn add_trait_bounds(mut generics: Generics, fields: &[NamedStructFieldWithAttr]) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            if need_bound(type_param, fields) {
                type_param.bounds.push(parse_quote!(::std::fmt::Debug));
            }
        }
    }
    generics
}

fn need_bound(tp: &TypeParam, fields: &[NamedStructFieldWithAttr]) -> bool {
  for field in  fields {
      if !is_phantom(&field.ty) && ty_contain_ident(&field.ty, &tp.ident) {
          return true
      }
  }
  false
}

fn derive_impl(input: &DeriveInput) -> Result<TokenStream, TokenStream> {
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
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let ret = match derive_impl(&input)  {
        Ok(d) => d,
        Err(e) =>e,
    };

    ret.into()
}
