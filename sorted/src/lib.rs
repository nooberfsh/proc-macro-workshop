#![feature(is_sorted)]

extern crate proc_macro;

use syn::*;
use syn::parse::{ParseStream, Parse};
use quote::*;
use proc_macro2::*;
use syn::fold::Fold;
use itertools::Itertools;
use syn::spanned::Spanned;

use std::cmp::{Ordering, PartialOrd};
use std::fmt::{self, Display, Formatter};

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

#[proc_macro_attribute]
pub fn check(args: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let _ = args;
    let input = parse_macro_input!(input as Item);

    match extract_fn(input.clone()) {
        Ok((f, e)) => if let Some(e) = e {
            let err = e.to_compile_error();
            let ret = quote!(#f #err);
            ret.into()
        } else  {
            let ret = quote!(#f);
            ret.into()
        }
        Err(e) => {
            let err = e.to_compile_error();
            let ret = quote!(#input #err);
            ret.into()
        }
    }
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

fn extract_fn(input: Item) -> Result<(ItemFn, Option<Error>)> {
    match input {
        Item::Fn(f) => Ok(trans_fn(f)),
        _ => Err(Error::new(Span::call_site(), "expected fn"))
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

struct MatchChecker {
    errors: Vec<Error>,
}

#[derive(PartialEq, Eq)]
struct MyPat {
    pat: Pat
}

impl MyPat {
    fn span(&self) -> Span {
        match &self.pat {
            Pat::TupleStruct(p) => p.path.span(),
            Pat::Path(p) => p.path.span(),
            Pat::Wild(p) => p.span(),
            t => {
                println!("span: {:?}", t);
                todo!()
            }
        }
    }
}


impl PartialOrd for MyPat {
    fn partial_cmp(&self, other: &MyPat) -> Option<Ordering> {
        match (&self.pat, &other.pat) {
            (Pat::TupleStruct(l), Pat::TupleStruct(r)) => {
                let l_ident = &l.path.segments.last().unwrap().ident;
                let r_ident = &r.path.segments.last().unwrap().ident;
                l_ident.partial_cmp(r_ident)
            },
            (Pat::Path(l), Pat::Path(r)) => {
                let l_ident = &l.path.segments.last().unwrap().ident;
                let r_ident = &r.path.segments.last().unwrap().ident;
                l_ident.partial_cmp(r_ident)
            },
            (Pat::Ident(a), Pat::Ident(b)) => {
                a.ident.partial_cmp(&b.ident)
            },
            (_, Pat::Wild(_)) => {
                Some(Ordering::Less)
            }
            (Pat::Wild(_), _) => {
                Some(Ordering::Greater)
            }
            t => {
                println!("partial_ord: {:?}", t);
                todo!()
            }
        }
    }
}

impl Display for MyPat {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match &self.pat {
            Pat::TupleStruct(p) => {
                let s = path_to_string(&p.path);
                write!(f, "{}", s)
            },
            Pat::Path(p) => {
                let s = path_to_string(&p.path);
                write!(f, "{}", s)
            }
            Pat::Wild(_) => {
                write!(f, "{}", "_")
            }
            //Pat::TupleStruct(p) => write!(f, "{}", p.path.segments.last().unwrap().ident),
            t => {
                println!("display {:?}", t);
                todo!()
            }
        }
    }
}

fn path_to_string(p: &Path) -> String  {
    let s: Vec<_> = p.segments.clone().into_iter().map(|ps| format!("{}", ps.ident))
                    .collect();
    format!("{}", s.join("::"))
}

impl MatchChecker {
    fn check(&mut self, m: &ExprMatch) {
        let ret: Vec<_> = m.arms.iter().map(|a| MyPat {pat: a.pat.clone()}).collect();

        let mut unsupported = vec![];
        for p in &ret {
            match &p.pat {
                Pat::Slice(s) => {
                    let e = Error::new(s.span(), format!("unsupported by #[sorted]"));
                    unsupported.push(e);
                    break
                }
                _ => {},
            }
        }

        if !unsupported.is_empty() {
            self.errors.extend(unsupported);
            return;
        }
        
        if ret.len() <= 1 { return }

        for i in 0..(ret.len() - 1) {
            if ret[i+1] < ret[i] {
                let bigger = find_bigger(&ret[0..=i], &ret[i+1]);
                let e = Error::new(ret[i+1].span(), format!("{} should sort before {}", ret[i+1], bigger));
                self.errors.push(e);
            }
        }
        
    }

    fn error(self) -> Option<Error> {
        self.errors.into_iter()
            .fold1(|mut l, r| {l.combine(r); l })
    }
}

impl Fold for MatchChecker {
    fn fold_expr_match(&mut self, i: ExprMatch) -> ExprMatch {
        let (attrs, removed) = remove_sorted_attr(&i.attrs);
        if removed {
            let mut ret = i.clone();
            ret.attrs = attrs;
            self.check(&ret);
            ret
        } else {
            i
        }
    }
}



fn remove_sorted_attr(attrs: &[Attribute]) -> (Vec<Attribute>, bool) {
    let mut ret = vec![];
    let mut removed  = false;

    for attr in attrs {
        let s: Path = parse_quote!(sorted);
        if attr.path == s {
            removed = true;
        } else {
            ret.push(attr.clone());
        }
    }

    (ret, removed)
}

fn trans_fn(f: ItemFn) -> (ItemFn, Option<Error>) {
    let mut matcher = MatchChecker {errors: vec![]};
    let ret = matcher.fold_item_fn(f);
    (ret, matcher.error())
}

fn find_bigger<'a, T: PartialOrd>(slice: &'a [T], id: &'a T) -> &'a T {
    assert!(slice.len() > 0);
    assert!(&slice[slice.len() - 1] > id);

    for i in (0..slice.len()).rev() {
        if &slice[i] > id && ( i == 0 || id >= &slice[i - 1] ) {
            return &slice[i]
        }
    }
    unreachable!()
}
