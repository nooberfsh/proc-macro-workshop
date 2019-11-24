extern crate proc_macro;

use syn::*;
use syn::parse::*;
use proc_macro2::{TokenStream, TokenTree, Literal, Group};


#[derive(Debug)]
struct Seq {
    ident: Ident,
    start: usize,
    end: usize,
    content: TokenStream,
}

enum State {
    Empty,
    Ident(Ident),
    IdentPound(Ident),
    IdentPoundNum(Ident),
    IdentPoundNumPound(Ident),
}

impl Seq {
    fn expand(&self) -> TokenStream {
        (self.start..self.end)
            .map(|n| replace(n, self.ident.clone(), self.content.clone()))
            .flatten()
            .collect()
    }

}

fn replace(n: usize, id: Ident, content: TokenStream) -> impl Iterator<Item = TokenTree> {
    content
        .clone()
        .into_iter()
        .map(move |tt| match tt {
            TokenTree::Ident(i) if i == id=> TokenTree::Literal(Literal::usize_unsuffixed(n)),
            TokenTree::Group(g) => {
                let grouped = g.stream();
                let delim = g.delimiter();
                let replaced = replace(n, id.clone(), grouped).collect();
                TokenTree::Group(Group::new(delim, replaced))
            },
            _ => tt,
        })
}

impl Parse for Seq {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident: Ident = input.parse()?;
        input.parse::<Token![in]>()?;
        let s: LitInt = input.parse()?;
        let start = s.base10_parse::<usize>()?;
        input.parse::<Token![..]>()?;
        let e: LitInt = input.parse()?;
        let end = e.base10_parse::<usize>()?;
        let c;
        braced!(c in input);
        let content : TokenStream = c.parse()?;
        // TODO: check range is valid
        Ok(Seq{ident, start, end, content})
    }
}

#[proc_macro]
pub fn seq(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let seq: Seq = parse_macro_input!(input);
    let ret = seq.expand();
    println!("{}", ret);
    ret.into()
}
