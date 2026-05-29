use std::{collections::BTreeMap, env, fs, iter::once, path::PathBuf};

const MORSE_DEFINITIONS: &str = "src/morse.txt";

/// Symbol on the wire
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Symbol {
    /// A dot
    DOT,
    /// A line
    LINE,
}
pub use Symbol::*;
use itertools::Itertools;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};

fn main() {
    cargo_emit::rerun_if_changed!(MORSE_DEFINITIONS);

    let definitions = fs::read_to_string(MORSE_DEFINITIONS)
        .expect("Cannot read definition file")
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
        .sorted_by_key(|(k, _)| *k)
        .dedup_by(|(a, _), (b, _)| a == b)
        .collect::<Vec<_>>();

    let is_morse_letter = is_morse_letter(&definitions);
    let encode = encode(&definitions);
    let decode = decode(&definitions);
    let generated = quote! {
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

fn is_morse_letter(definitions: &[(u8, Vec<Symbol>)]) -> TokenStream {
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

fn encode(definitions: &[(u8, Vec<Symbol>)]) -> TokenStream {
    let definitions = definitions.iter().map(|(c, s)| {
        let s = s.iter().map(|s| match s {
            DOT => quote! { Symbol::DOT },
            LINE => quote! { Symbol::LINE },
        });
        quote! {
            #c => &nunny::slice![ #( #s ),* ]
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

fn decode(definitions: &[(u8, Vec<Symbol>)]) -> TokenStream {
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

    struct StateData {
        name: Ident,
        value: Option<u8>,
    }

    let mut states = BTreeMap::from([(
        vec![],
        StateData {
            name: format_ident!("Empty"),
            value: None,
        },
    )]);
    for (value, symbols) in definitions {
        for i in 1..symbols.len() {
            states
                .entry(symbols[..i].to_vec())
                .or_insert_with_key(|key| StateData {
                    name: variant_name(key),
                    value: None,
                });
        }
        let entry = StateData {
            name: variant_name(symbols),
            value: Some(*value),
        };
        states.insert(symbols.clone(), entry);
    }

    let variants = states.values().map(|v| &v.name);
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
    let symbols_arms = states.iter().map(|(symbols, StateData { name, .. })| {
        let symbols = symbols.iter().map(|s| match s {
            DOT => quote! { Symbol::DOT },
            LINE => quote! { Symbol::LINE },
        });
        quote! { Self::#name => &[ #( #symbols ),* ]}
    });
    let finish_arms = states.values().filter_map(|StateData { name, value }| {
        value.map(|value| quote! { Self::#name => Some(#value) })
    });

    quote! {
        #[derive(Clone, Copy)]
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
