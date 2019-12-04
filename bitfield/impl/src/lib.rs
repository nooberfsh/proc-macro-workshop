extern crate proc_macro;

use syn::*;
use quote::*;
use syn::spanned::Spanned;
use proc_macro2::*;

#[proc_macro_derive(BitfieldSpecifier)]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let data = parse_macro_input!(input as DeriveInput);

    match derive_impl(data) {
        Ok(d) => d.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn derive_impl(data: DeriveInput) -> Result<TokenStream> {
    let span = Span::call_site();
    let name = data.ident;
    let data = match data.data {
        Data::Enum(de) => de,
        _ =>  return Err(Error::new(span, "must be enum")),
    };

    let len = data.variants.len();

    let bit_size = if let Some(d) = bit_size(len){
        d
    } else {
        return Err(Error::new(span, "BitfieldSpecifier expected a number of variants which is a power of 2"))
    };

    let booleans = data.variants.iter().map(|v|{
        let v_name = &v.ident;
        let span = v.span();
        quote_spanned! { span=>
            ((#name::#v_name as usize) < #len) as usize
        }
    });

    let assert_trait = data.variants.iter().map(|v|{
        format_ident!("Assert{}", v.ident)
    });

    let ret = quote! {
        impl ::bitfield::Specifier for #name {
            const BITS: usize = #bit_size;
            const SIZE: usize = ::std::mem::size_of::<#name>();
            type Container = #name;

            fn get(buf: &[u8], buf_idx: usize) -> #name {
                let mut ret: [u8; Self::SIZE]= [0u8; Self::SIZE];
                let mut left_bits = Self::BITS;
                let mut start = buf_idx;

                for i in 0..Self::SIZE {
                    let read_size = if left_bits > 8 { 8 } else { left_bits };
                    let b = crate::get_byte(buf, start, read_size);
                    ret[i] = b;
                    left_bits -= read_size;
                    start += read_size;
                }
                unsafe {::std::mem::transmute(ret)}
            }

            fn set(buf: &mut [u8], buf_idx: usize, data: #name) {
                let data: [u8; Self::SIZE] = unsafe {  ::std::mem::transmute(data)  };
                
                let mut left_bits = Self::BITS;
                let mut start = buf_idx;

                for i in 0..Self::SIZE {
                    let write_size = if left_bits > 8 { 8 } else { left_bits };
                    crate::set_byte(buf, start, data[i], write_size);
                    left_bits -= write_size;
                    start += write_size;
                }
            }
        }

        #(
                trait #assert_trait: ::bitfield::checks::DiscriminantInRange {}
                impl #assert_trait for  <[u8; #booleans] as ::bitfield::checks::Array2>::Content {}
        )*
    };

    Ok(ret.into())
}


//////////////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////////////


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

    let mut assert_attrs = vec![];
    for f in &fields {
        let ty = &f.ty;
        for attr in &f.attrs {
            let meta = attr.parse_meta()?;
            if let Meta::NameValue(nv) = meta {
                if let Some(i) = nv.path.get_ident() {
                    if i == "bits" {
                        let lit = &nv.lit;
                        let span = lit.span();

                        let ret = quote_spanned! { span=>
                            let _: [(); #lit] = [(); <#ty as Specifier>::BITS];
                        };
                        assert_attrs.push(ret);
                    }
                }
            }
        }
    }

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
                <#ty as Specifier>::set(&mut self.data, buf_idx, data)
            }

            fn #getter(&self) -> <#ty as Specifier>::Container {
                let buf_idx = self.#idx_f_name();
                <#ty as Specifier>::get(&self.data, buf_idx)
            }

            #idx_fn
        }
    }).collect();

    let name = &s.ident;

    let size = quote! {
        (#(<#tyes as Specifier>::BITS)+*) / 8
    };

    let is_multiple8 = quote! {
        (#(<#tyes as Specifier>::BITS)+*) % 8
    };

    let ret = quote! {
        #[derive(Debug)]
        #[repr(C)]
        pub struct #name {
            data: [u8; #size],
        }


        trait AssertMultiple8: ::bitfield::checks::TotalSizeIsMultipleOfEightBits {}
        impl AssertMultiple8 for  <[u8; #is_multiple8] as ::bitfield::checks::Array>::Content {}

        fn _assert() {
            #(
                #assert_attrs
            )*
        }

        impl #name {
            pub fn new() -> Self {
                #name {
                    data: [0u8; #size],
                }
            }

            #(#get_set )*
        }
    };

    Ok(ret)
}

fn bit_size(mut p: usize) -> Option<usize> {
    // is power of 2
    if (p != 0) && ((p & (p - 1)) == 0) {
        let mut ret = 0;
        while p != 1 {
            ret += 1;
            p = p >> 1;
        }
        Some(ret)
    } else {
        None
    }
}
