use std::{
    cmp::{Reverse, min},
    collections::{BTreeMap, BTreeSet},
    env, fs,
    iter::once,
    path::PathBuf,
    range::Range,
};

const MORSE_DEFINITIONS: &str = "src/morse.txt";

#[path = "src/encoding/symbol.rs"]
mod symbol;
use symbol::*;

use itertools::Itertools;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};

fn main() {
    cargo_emit::rerun_if_changed!(MORSE_DEFINITIONS);

    let definitions = fs::read_to_string(MORSE_DEFINITIONS).expect("Cannot read definition file");
    let definitions = definitions
        .lines()
        .flat_map(|line| {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                return None;
            }

            let Some((ch, def)) = line.split_once(' ') else {
                panic!("All non-empty or comment lines should contain a space")
            };
            assert_eq!(1, ch.len(), "A single char should be before the space");
            let ch = ch.as_bytes()[0];
            if ch != ch.to_ascii_uppercase() {
                cargo_emit::warning!(
                    "Letters are automatically made uppercase, `{}` won't be matched",
                    ch as char
                )
            }

            let symbols = def
                .trim_start()
                .chars()
                .map(|ch| match ch {
                    '.' => DOT,
                    '-' => LINE,
                    _ => panic!("`{ch}` is not a valid morse symbol"),
                })
                .collect::<Vec<_>>();

            Some((ch, symbols))
        })
        .unique_by(|(k, _)| *k);

    let (symbols_slice, definitions) = build_symbols_slice(definitions);

    let static_symbols_slice = static_symbols_slice(&symbols_slice);
    let is_morse_letter = is_morse_letter(&definitions);
    let encode = encode(&definitions);
    let decode = decode(&definitions, &symbols_slice);
    let generated = quote! {
        #static_symbols_slice
        #is_morse_letter
        #encode
        #decode
    };

    let out = PathBuf::from(env::var("OUT_DIR").expect("Cargo should set OUT_DIR"))
        .join("morse.rs")
        .to_str()
        .unwrap()
        .to_owned();

    fs::write(&out, generated.to_string()).expect("Cannot write generated code");
    cargo_emit::rustc_env!("MORSE_GENERATED_RS", "{}", out)
}

/// Build a supersequence to use less memory
fn build_symbols_slice(
    definitions: impl IntoIterator<Item = (u8, Vec<Symbol>)>,
) -> (Vec<Symbol>, Vec<(u8, Range<usize>)>) {
    let mut supersequence = vec![];
    let mut ranges = vec![];

    for (ch, mut symbols) in definitions
        .into_iter()
        // Taking longer sequences first, so subsequences are guaranteed to
        // arrive later
        .sorted_by_key(|(_, s)| Reverse(s.len()))
    {
        if let Some((start, _)) = supersequence
            .windows(symbols.len())
            .enumerate()
            .find(|(_, window)| window == &symbols)
        {
            let end = start + symbols.len();

            ranges.push((ch, (start..end).into()));
        } else {
            let overlap = (0..=min(symbols.len(), supersequence.len()))
                .rev()
                .find(|overlap| {
                    &supersequence[supersequence.len() - overlap..] == &symbols[..*overlap]
                })
                .unwrap_or(0);

            let start = supersequence.len() - overlap;
            supersequence.extend_from_slice(&mut symbols[overlap..]);
            let end = supersequence.len();

            ranges.push((ch, (start..end).into()));
        }
    }

    (supersequence, ranges)
}

fn static_symbols_slice(symbols_slice: &[Symbol]) -> TokenStream {
    let len = symbols_slice.len();
    let symbols_slice = symbols_slice.into_iter().map(|s| match s {
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

fn is_morse_letter(definitions: &[(u8, Range<usize>)]) -> TokenStream {
    let defined = definitions.iter().map(|(c, _)| c);
    quote! {
        pub(super) const fn is_morse_letter(ch: u8) -> bool {
            match ch {
                #( #defined  ) | * => true,
                _ => false
            }
        }
    }
}

fn encode(definitions: &[(u8, Range<usize>)]) -> TokenStream {
    let definitions = definitions.iter().map(|(c, Range { start, end })| {
        quote! {
            #c => nunny::Slice::new(&SYMBOLS_SLICE[ #start .. #end  ]).unwrap()
        }
    });
    quote! {
        pub(super) const fn encode(ch: u8) -> &'static nunny::Slice<Symbol> {
            match ch {
                #( #definitions  , )*
                _ => unreachable!()
            }
        }
    }
}
fn variant_name(symbols: &[Symbol]) -> Ident {
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

fn decode(definitions: &[(u8, Range<usize>)], symbols_slice: &[Symbol]) -> TokenStream {
    struct StateData {
        name: Ident,
        value: Option<u8>,
        range: Range<usize>,
        variant: Option<u8>,
    }

    let mut states = BTreeMap::from([(
        vec![],
        StateData {
            name: format_ident!("Empty"),
            value: None,
            range: (0..0).into(),
            variant: None,
        },
    )]);
    for (value, range) in definitions {
        let symbols = &symbols_slice[*range];

        for i in 1..symbols.len() {
            states
                .entry(symbols[..i].to_vec())
                .or_insert_with_key(|key| StateData {
                    name: variant_name(key),
                    value: None,
                    range: (0..i).into(),
                    variant: None,
                });
        }
        let entry = StateData {
            name: variant_name(symbols),
            value: Some(*value),
            range: *range,
            variant: Some(*value),
        };
        states.insert(symbols.to_owned(), entry);
    }

    if states.len() > 256 {
        panic!("Too many states - #[repr(u8)] is going to fail")
    }

    {
        let assigned = BTreeSet::from_iter(
            states
                .values()
                .filter_map(|StateData { variant, .. }| *variant),
        );
        let mut highest_unassigned = 0u8;

        for unassigned in states
            .values_mut()
            .filter_map(|StateData { variant, .. }| variant.is_none().then_some(variant))
        {
            while assigned.contains(&highest_unassigned) {
                highest_unassigned += 1
            }
            *unassigned = Some(highest_unassigned);
            highest_unassigned += 1
        }
    }

    let states = states;

    let variants = states.values().map(|v| {
        let name = &v.name;
        let variant = v.variant.unwrap();
        quote! { #name = #variant }
    });
    let digest_arms = states.iter().flat_map(
        |(
            symbols,
            StateData {
                name: current_state,
                ..
            },
        )| {
            [Symbol::DOT, Symbol::LINE]
                .into_iter()
                .filter_map(|next| {
                    let next_symbols = symbols
                        .iter()
                        .copied()
                        .chain(once(next))
                        .collect::<Vec<_>>();
                    let next = match next {
                        DOT => quote! { Symbol::DOT },
                        LINE => quote! { Symbol::LINE },
                    };
                    states.get(&next_symbols).map(
                        |StateData {
                             name: next_state, ..
                         }| {
                            quote! {
                                (Self::#current_state, #next) => Some( Self::#next_state )
                            }
                        },
                    )
                })
                .collect::<Vec<_>>()
        },
    );
    let symbols_arms = states.iter().map(
        |(
            _,
            StateData {
                name,
                range: Range { start, end },
                ..
            },
        )| {
            quote! { Self::#name => &SYMBOLS_SLICE[ #start .. #end  ]}
        },
    );
    let finish_arms = states.values().filter_map(|StateData { name, value, .. }| {
        value.map(|value| quote! { Self::#name => Some(#value) })
    });

    quote! {
        #[derive(Clone, Copy)]
        #[repr(u8)]
        pub(super) enum DecodeState {
            #(#variants),*
        }

        impl DecodeState {
            pub const fn new() -> Self {
                Self::Empty
            }

            pub const fn is_empty(&self) -> bool {
                matches!(self, Self::Empty)
            }

            pub const fn symbols(&self) -> &'static [Symbol] {
                match self {
                    #( #symbols_arms ),*
                }
            }

            pub const fn digest(self, s: Symbol) -> Option<Self> {
                match (self, s) {
                    #( #digest_arms , )*
                    _ => None
                }
            }

            pub const fn finish(self) -> Option<u8> {
                match self {
                    #( #finish_arms , )*
                    _ => None
                }
            }
        }
    }
}
