extern crate proc_macro;

use syn::*;
use syn::parse::*;
use proc_macro2::{TokenStream, TokenTree, Literal, Group};
use quote::*;

#[derive(Debug)]
struct Seq {
    ident: Ident,
    start: usize,
    end: usize,
    content: TokenStream,
}

// ident (#N)+ (# ident)?
#[derive(Clone)]
enum StateKind {
    Empty,
    Ident(Ident),
    IdentPound(Ident),
    IdentPoundNum(Ident, usize),
    IdentPoundNumPound(Ident, usize),
}

struct State {
    ident: Ident,
    num: usize,
    kind: StateKind,
}

impl State {
    fn new(ident: Ident, num: usize) -> State {
        State {
            ident,
            num,
            kind: StateKind::Empty
        }
    }

    fn finalize(&mut self) -> Result<Vec<TokenTree>> {
        let ret = match self.kind.clone() {
            StateKind::Empty => Ok(vec![]),
            StateKind::Ident(i) => Ok(vec![i.into()]),
            StateKind::IdentPound(i) => Err(syn::Error::new(i.span().unwrap().into(), "need N")),
            StateKind::IdentPoundNum(i, c) => Ok(vec![gen_ident(&i,  self.num, c).into()]),
            StateKind::IdentPoundNumPound(i, _) => Err(syn::Error::new(i.span().unwrap().into(), "need N or ident")),
        };
        self.clear();
        ret
    }

    fn clear(&mut self) {
        self.kind = StateKind::Empty;
    }

    fn accept(&mut self, stream: TokenStream) -> Result<TokenStream> {
        let mut ret = vec![];

        for tt in stream {
            if let Some(tt)  = self.accept_tt(tt)? {
                ret.extend(tt);
            }
        }

        let f = self.finalize()?;
        ret.extend(f);

        Ok(ret.into_iter().collect())
    }

    fn accept_tt(&mut self, tt: TokenTree) -> Result<Option<Vec<TokenTree>>> {
        match self.kind.clone() {
            StateKind::Empty => {
                match tt {
                    TokenTree::Ident(i) if i == self.ident => {
                        let l = Literal::usize_unsuffixed(self.num);
                        Ok(Some(vec![l.into()]))
                    }
                    TokenTree::Ident(i) => {
                        self.kind =  StateKind::Ident(i);
                        Ok(None)
                    }
                    TokenTree::Group(g) => {
                        let dim = g.delimiter();
                        let mut new_state = State::new(self.ident.clone(), self.num);
                        let stream = new_state.accept(g.stream())?;
                        let t = Group::new(dim, stream);
                        Ok(Some(vec![t.into()]))
                    },
                    _ => Ok(Some(vec![tt])),
                }
            },
            StateKind::Ident(o) => {
                match tt {
                    TokenTree::Ident(i) => if i == self.ident {
                        let ret = TokenTree::Literal(Literal::usize_unsuffixed(self.num));
                        self.clear();
                        Ok(Some(vec![o.into(), ret]))
                    }
                    else {
                        self.kind = StateKind::Ident(i);
                        Ok(Some(vec![o.into()]))
                    },
                    TokenTree::Group(g) => {
                        let dim = g.delimiter();
                        let mut new_state = State::new(self.ident.clone(), self.num);
                        let stream = new_state.accept(g.stream())?;
                        let t = Group::new(dim, stream);
                        self.clear();
                        Ok(Some(vec![o.into(), t.into()]))
                    },
                    TokenTree::Punct(p) => if p.as_char() == '#' {
                        self.kind = StateKind::IdentPound(o);
                        Ok(None)
                    } else {
                        let ret = TokenTree::Punct(p);
                        self.clear();
                        Ok(Some(vec![o.clone().into(), ret]))
                    }
                    TokenTree::Literal(l) => {
                        let ret = TokenTree::Literal(l);
                        self.kind = StateKind::Empty;
                        Ok(Some(vec![o.clone().into(), ret]))
                    }
                }
            },
            StateKind::IdentPound(o) => {
                match tt {
                    TokenTree::Ident(i)  if i == self.ident => {
                      self.kind = StateKind::IdentPoundNum(o, 1);
                        Ok(None)
                    },
                    _ => Err(syn::Error::new(tt.span(), "need N")),
                }

            },
            StateKind::IdentPoundNum(o,c ) => {
                match tt {
                    TokenTree::Ident(i)  if i == self.ident => {
                        let t = TokenTree::Literal(Literal::usize_unsuffixed(self.num));
                        let g =  gen_ident(&o, self.num, c);
                        Ok(Some(vec![g.into(), t]))
                    },
                    TokenTree::Punct(p) if p.as_char() == '#' => {
                        self.kind = StateKind::IdentPoundNumPound(o.clone(), c);
                        Ok(None)
                    },
                    TokenTree::Group(g) => {
                        let dim = g.delimiter();
                        let mut new_state = State::new(self.ident.clone(), self.num);
                        let stream = new_state.accept(g.stream())?;
                        let t = Group::new(dim, stream);
                        let o = gen_ident(&o, self.num, c);
                        self.clear();
                        Ok(Some(vec![o.into(), t.into()]))
                    },
                    _ =>  {
                        let g = gen_ident(&o, self.num, c);
                        self.clear();
                        Ok(Some(vec![g.into(), tt]))
                    }
                }
            },
            StateKind::IdentPoundNumPound(o, c) => {
                match tt {
                    TokenTree::Ident(i)  => if i == self.ident {
                        self.kind = StateKind::IdentPoundNum(i.clone(), c + 1);
                        Ok(None)
                    } else {
                        let g = gen_ident_tail(&o, self.num, c, &i);
                        self.clear();
                        Ok(Some(vec![g.into()]))
                    },
                    _ => Err(syn::Error::new(tt.span(), "need ident or N")),
                }
            },
        }
    }
}

fn gen_ident(ident: &Ident, n: usize, count: usize) -> Ident {
    assert!(count > 0);
    let mut new_num = 0;
    for i in 0..count {
        new_num += 10_usize.pow(i as u32) * n;
    }
    format_ident!("{}{}", ident, new_num)
}

fn gen_ident_tail(ident: &Ident, n: usize, count: usize, tail: &Ident) -> Ident {
    let tmp = gen_ident(ident, n, count);
    format_ident!("{}{}", tmp, tail)
}

impl Seq {
    fn expand(&self) -> Result<TokenStream> {
        let mut ret = vec![];
        for i in self.start..self.end {
            let mut state = State::new(self.ident.clone(), i);
            let stream = state.accept(self.content.clone())?;
            //println!("num: {}, stream: {}", i, stream);
            ret.push(stream)
        }
        Ok(ret.into_iter().collect())
    }
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
    //println!("{}", seq.content);
    match seq.expand() {
        Ok(d) => d.into(),
        Err(e) => e.to_compile_error().into(),
    }
}
