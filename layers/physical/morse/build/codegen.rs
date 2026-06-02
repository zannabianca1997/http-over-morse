use std::collections::BTreeMap;
use std::iter::{self, once};

use itertools::Itertools;
use proc_macro2::{Ident, Literal, TokenStream};
use quote::{format_ident, quote};

use crate::build::grammar::{Letters, Prosign};
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

pub fn enums(letters: &Letters, prosigns: &BTreeMap<String, Prosign>) -> TokenStream {
    let letter_variants = letters
        .clone()
        .into_iter()
        .map(|(ch, symbols)| {
            let ch = ch.get();
            let name = format_ident!("Letter{}", ch as char);
            let doc = format!(
                "Letter {} ({})",
                ch as char,
                symbols
                    .into_iter()
                    .map(|s| match s {
                        DOT => '.',
                        LINE => '-',
                    })
                    .collect::<String>()
            );
            let ch = Literal::byte_character(ch);
            quote! {
                #[doc= #doc]
                #name = #ch
            }
        })
        .collect_vec();
    let letter_values = letters
        .clone()
        .into_iter()
        .map(|(c, _)| c.get())
        .collect_vec();

    let prosign_variants = prosigns
        .iter()
        .map(|(name, Prosign { code, maps_to })| {
            let maps_to = maps_to.get();
            let name = format_ident!("{}", &stringcase::pascal_case(name));
            let doc = format!("Prosign {code}");
            let ch = Literal::byte_character(maps_to);
            quote! {
                #[doc= #doc]
                #name = #ch
            }
        })
        .collect_vec();
    let prosign_values = prosigns
        .values()
        .map(|Prosign { maps_to, .. }| maps_to.get())
        .collect_vec();

    let space_variants = once(quote! {
        // Space
        Space = b' '
    })
    .collect_vec();
    let space_values = vec![b' '];

    let letter_or_prosign_variants =
        iter::chain(letter_variants.clone(), prosign_variants.clone()).collect_vec();
    let letter_or_prosign_values =
        iter::chain(letter_values.clone(), prosign_values.clone()).collect_vec();

    let letter_or_space_variants =
        iter::chain(letter_variants.clone(), space_variants.clone()).collect_vec();
    let letter_or_space_values =
        iter::chain(letter_values.clone(), space_values.clone()).collect_vec();

    let letter_or_prosign_or_space_variants =
        iter::chain(letter_or_prosign_variants.clone(), space_variants.clone()).collect_vec();
    let letter_or_prosign_or_space_values =
        iter::chain(letter_or_prosign_values.clone(), space_values.clone()).collect_vec();

    quote! {
        /// A valid morse code letter
        #[derive(Clone, Copy, PartialEq, Eq)]
        #[repr(u8)]
        pub enum Letter {
            #( #letter_variants ), *
        }
        impl Letter {
            pub const fn new(value: u8) -> Option<Self> {
                match value {
                    #( #letter_values ) | * => Some(unsafe { core::mem::transmute(value) }),
                    _ => None
                }
            }
        }

        /// A morse code prosign
        #[derive(Clone, Copy)]
        #[repr(u8)]
        pub enum Prosign {
            #( #prosign_variants ), *
        }
        impl Prosign {
            pub const fn new(value: u8) -> Option<Self> {
                match value {
                    #( #prosign_values ) | * => Some(unsafe { core::mem::transmute(value) }),
                    _ => None
                }
            }
        }

        /// A morse code space between words
        #[derive(Clone, Copy)]
        #[repr(u8)]
        pub enum WordSpace {
            #( #space_variants ), *
        }
        impl WordSpace {
            pub const fn new(value: u8) -> Option<Self> {
                match value {
                    #( #space_values ) | * => Some(unsafe { core::mem::transmute(value) }),
                    _ => None
                }
            }
        }

        /// A morse code letter or prosign
        #[derive(Clone, Copy)]
        #[repr(u8)]
        pub enum LetterOrProsign {
            #( #letter_or_prosign_variants ), *
        }
        impl LetterOrProsign {
            pub const fn new(value: u8) -> Option<Self> {
                match value {
                    #( #letter_or_prosign_values ) | * => Some(unsafe { core::mem::transmute(value) }),
                    _ => None
                }
            }
        }

        /// A morse code letter or space
        #[derive(Clone, Copy)]
        #[repr(u8)]
        pub enum LetterOrSpace {
            #( #letter_or_space_variants ), *
        }
        impl LetterOrSpace {
            pub const fn new(value: u8) -> Option<Self> {
                match value {
                    #( #letter_or_space_values ) | * => Some(unsafe { core::mem::transmute(value) }),
                    _ => None
                }
            }
        }

        /// A morse code letter or prosign or space
        #[derive(Clone, Copy)]
        #[repr(u8)]
        pub enum LetterOrProsignOrSpace {
            #( #letter_or_prosign_or_space_variants ), *
        }
        impl LetterOrProsignOrSpace {
            pub const fn new(value: u8) -> Option<Self> {
                match value {
                    #( #letter_or_prosign_or_space_values ) | * => Some(unsafe { core::mem::transmute(value) }),
                    _ => None
                }
            }
        }
    }
}
