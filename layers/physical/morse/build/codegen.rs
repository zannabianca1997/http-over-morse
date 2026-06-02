use std::num::NonZero;

use itertools::Itertools;
use proc_macro2::{Ident, Literal, TokenStream};
use quote::{format_ident, quote};

use crate::src::symbol::Symbol;
use crate::src::symbol::Symbol::*;

pub fn static_symbols_slice(symbols_slice: &[Symbol]) -> TokenStream {
    let len = symbols_slice.len();
    let symbols_slice = symbols_slice.iter().map(|s| match s {
        DOT => quote! { Symbol::DOT },
        LINE => quote! { Symbol::LINE },
    });

    quote! {
        /// Supersequence of symbols, guaranteed to contains all morse sequences as subsequences
        static SYMBOLS_SLICE: [ Symbol ; #len ] = [
            #( #symbols_slice ),*
        ];
    }
}

pub fn is_morse_letter(definitions: &[(NonZero<u8>, std::range::Range<usize>)]) -> TokenStream {
    let defined = definitions
        .iter()
        .map(|(c, _)| Literal::byte_character(c.get()));
    quote! {
        pub(super) const fn is_morse_letter(ch: u8) -> bool {
            match ch {
                #( #defined  ) | * => true,
                _ => false
            }
        }
    }
}

pub fn variant_name(symbols: &[Symbol]) -> Ident {
    format_ident!(
        "{}",
        symbols
            .iter()
            .map(|s| match s {
                LINE => "Line",
                DOT => "Dot",
            })
            .join("")
    )
}
