#![doc = include_str!("../README.md")]

use std::collections::BTreeSet;

use either::Either::{Left, Right};
use itertools::Itertools;
use morse_symbol::Symbol;
use proc_macro2::{Ident, Literal};
use quote::{ToTokens, format_ident, quote};

static LETTERS: &[(u8, &'static [u8])] = &[
    (b'A', b".-"),
    (b'B', b"-..."),
    (b'C', b"-.-."),
    (b'D', b"-.."),
    (b'E', b"."),
    (b'F', b"..-."),
    (b'G', b"--."),
    (b'H', b"...."),
    (b'I', b".."),
    (b'J', b".---"),
    (b'K', b"-.-"),
    (b'L', b".-.."),
    (b'M', b"--"),
    (b'N', b"-."),
    (b'O', b"---"),
    (b'P', b".--."),
    (b'Q', b"--.-"),
    (b'R', b".-."),
    (b'S', b"..."),
    (b'T', b"-"),
    (b'U', b"..-"),
    (b'V', b"...-"),
    (b'W', b".--"),
    (b'X', b"-..-"),
    (b'Y', b"-.--"),
    (b'Z', b"--.."),
    (b'0', b"-----"),
    (b'1', b".----"),
    (b'2', b"..---"),
    (b'3', b"...--"),
    (b'4', b"....-"),
    (b'5', b"....."),
    (b'6', b"-...."),
    (b'7', b"--..."),
    (b'8', b"---.."),
    (b'9', b"----."),
];

pub struct Item {
    group: GroupData,
    name: Ident,
    value: u8,
}

enum GroupData {
    Letter {
        symbols: Vec<Symbol>,
        base32: Option<u8>,
        ascii: u8,
        ascii_aliases: Vec<u8>,
    },
    WordSpace,
    Prosign {
        symbols: Vec<Symbol>,
        code: &'static [u8],
        ascii: u8,
    },
}
pub enum Group {
    Letter,
    WordSpace,
    Prosign,
}
impl Group {
    fn combinations() -> &'static [&'static [Self]] {
        use Group::*;
        &[
            &[Letter],
            &[WordSpace],
            &[Prosign],
            &[Letter, WordSpace],
            &[WordSpace, Prosign],
            &[Letter, Prosign],
            &[Letter, WordSpace, Prosign],
        ]
    }

    fn complement(combination: & [Self]) -> Option<&'static [Self]> {
        use Group::*;
        Some(match combination {
            &[Letter] => &[WordSpace, Prosign],
            &[WordSpace] => &[Letter, Prosign],
            &[Prosign] => &[Letter, WordSpace],
            &[Letter, WordSpace] => &[Prosign],
            &[WordSpace, Prosign] => &[Letter],
            &[Letter, Prosign] => &[WordSpace],
            &[Letter, WordSpace, Prosign] => return None,
            _ => panic!(),
        })
    }

    fn supersets(combination: & [Self]) -> &'static [&'static [Self]] {
        use Group::*;
        match combination {
            &[Letter] => &[
                &[Letter, WordSpace],
                &[Letter, Prosign],
                &[Letter, WordSpace, Prosign],
            ],
            &[WordSpace] => &[
                &[Letter, WordSpace],
                &[WordSpace, Prosign],
                &[Letter, WordSpace, Prosign],
            ],
            &[Prosign] => &[
                &[Letter, Prosign],
                &[WordSpace, Prosign],
                &[Letter, WordSpace, Prosign],
            ],
            &[Letter, WordSpace] | &[WordSpace, Prosign] | &[Letter, Prosign] => {
                &[&[Letter, WordSpace, Prosign]]
            }
            &[Letter, WordSpace, Prosign] => &[],
            _ => panic!(),
        }
    }
    fn subsets(combination: & [Self]) -> &'static [&'static [Self]] {
        use Group::*;
        match combination {
            &[Letter] | &[WordSpace] | &[Prosign] => &[],
            &[Letter, WordSpace] => &[&[Letter], &[WordSpace]],
            &[WordSpace, Prosign] => &[&[WordSpace], &[Prosign]],

            &[Letter, Prosign] => &[&[Letter], &[Prosign]],
            &[Letter, WordSpace, Prosign] => &[
                &[Letter],
                &[WordSpace],
                &[Prosign],
                &[Letter, WordSpace],
                &[WordSpace, Prosign],
                &[Letter, Prosign],
            ],
            _ => panic!(),
        }
    }

    fn combination_name(combination: &[Self]) -> Ident {
        let name = combination
            .iter()
            .map(|c| match c {
                Group::Letter => "Letter",
                Group::WordSpace => "WordSpace",
                Group::Prosign => "Prosign",
            })
            .join("Or");

        format_ident!("{name}")
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Encoding {
    Base32,
    Ascii,
}

impl Item {
    /// All morse letters and digits
    fn letters() -> impl IntoIterator<Item = Self> {
        (b'A'..=b'Z')
            .map(|ch| Self {
                group: GroupData::Letter {
                    base32: Some(ch - b'A'),
                    ascii: ch,
                    ascii_aliases: vec![],
                    symbols: code(ch),
                },
                name: format_ident!("L_{}", ch as char),
                value: 0,
            })
            .chain((b'0'..=b'9').map(|ch| Self {
                group: GroupData::Letter {
                    base32: Some(ch - b'0' + b'Z' - b'A' + 1),
                    ascii: ch,
                    ascii_aliases: vec![],
                    symbols: code(ch),
                },
                name: format_ident!("D_{}", ch as char),
                value: 0,
            }))
            .map(|mut item| {
                let GroupData::Letter {
                    base32,
                    ascii,
                    ascii_aliases,
                    ..
                } = &mut item.group
                else {
                    unreachable!()
                };

                // Add aliases
                if let lower = ascii.to_ascii_lowercase()
                    && lower != *ascii
                {
                    ascii_aliases.push(lower);
                }
                if let upper = ascii.to_ascii_uppercase()
                    && upper != *ascii
                {
                    ascii_aliases.push(upper);
                }

                // Remove requirements for letters over base32 alphabet
                if base32.is_some_and(|v| v >= 32) {
                    *base32 = None
                }

                item
            })
    }

    /// Morse word space
    fn word_space() -> impl IntoIterator<Item = Self> {
        [Self {
            name: format_ident!("WordSpace"),
            group: GroupData::WordSpace,
            value: 0,
        }]
    }

    /// Morse prosigns
    fn prosigns() -> impl IntoIterator<Item = Self> {
        [
            Self {
                name: format_ident!("Attention"),
                group: GroupData::Prosign {
                    symbols: vec![],
                    code: b"KA",
                    ascii: 1,
                },
                value: 0,
            },
            Self {
                name: format_ident!("Break"),
                group: GroupData::Prosign {
                    symbols: vec![],
                    code: b"BT",
                    ascii: 2,
                },
                value: 0,
            },
            Self {
                name: format_ident!("Out"),
                group: GroupData::Prosign {
                    symbols: vec![],
                    code: b"AR",
                    ascii: 4,
                },
                value: 0,
            },
            Self {
                name: format_ident!("Aknowledge"),
                group: GroupData::Prosign {
                    symbols: vec![],
                    code: b"VE",
                    ascii: 6,
                },
                value: 0,
            },
            Self {
                name: format_ident!("NegativeAknowledge"),
                group: GroupData::Prosign {
                    symbols: vec![],
                    code: b"INT",
                    ascii: 25,
                },
                value: 0,
            },
        ]
        .into_iter()
        .map(|mut i| {
            let GroupData::Prosign {
                symbols, code: cd, ..
            } = &mut i.group
            else {
                unreachable!()
            };
            for ch in cd.into_iter() {
                symbols.append(&mut code(*ch));
            }
            i
        })
    }

    /// Iter all prosigns with the given encoding, inside the given groups
    fn iter(encoding: Encoding, groups: &[Group]) -> impl IntoIterator<Item = Self> {
        // Iter all, as the numeration must be consistent
        let all = Self::letters()
            .into_iter()
            .chain(Self::word_space())
            .chain(Self::prosigns());

        let numbered = match encoding {
            Encoding::Base32 => Right({
                let mut unassigned = 0;
                let mut assigned = BTreeSet::new();
                all.map(move |mut item| {
                    let value = match item.group {
                        GroupData::Letter { base32, .. } => base32,
                        GroupData::Prosign { .. } | GroupData::WordSpace => None,
                    };
                    item.value = value.unwrap_or_else(|| {
                        while assigned.contains(&unassigned) {
                            unassigned += 1
                        }
                        unassigned
                    });
                    assigned.insert(item.value);
                    item
                })
            }),
            Encoding::Ascii => Left(all.map(|mut item| {
                let value = match item.group {
                    GroupData::Letter { ascii, .. } | GroupData::Prosign { ascii, .. } => ascii,
                    GroupData::WordSpace => b' ',
                };
                item.value = value;
                item
            })),
        };

        numbered.filter(|n| {
            groups.iter().any(|g| match g {
                Group::Letter => n.is_letter(),
                Group::WordSpace => n.is_word_space(),
                Group::Prosign => n.is_prosign(),
            })
        })
    }

    fn is_letter(&self) -> bool {
        matches!(self.group, GroupData::Letter { .. })
    }
    fn is_word_space(&self) -> bool {
        matches!(self.group, GroupData::WordSpace)
    }
    fn is_prosign(&self) -> bool {
        matches!(self.group, GroupData::Prosign { .. })
    }

    fn variant(&self) -> impl ToTokens + use<> {
        let Self { name, value, .. } = self;
        let value = Literal::byte_character(*value);
        quote! { #name = #value }
    }

    fn r#enum(encoding: Encoding, groups: &[Group]) -> impl ToTokens {
        let name = Group::combination_name(groups);
        let variants = Self::iter(encoding, groups)
            .into_iter()
            .map(|v| v.variant());

        let from_impls = Group::subsets(groups).iter().map(|subset| {
            let subset = Group::combination_name(subset);
            quote! {
                impl From< #subset > for #name {
                    fn from(value: #subset) -> Self {
                        unsafe { ::core::mem::transmute(value) }
                    }
                }
            }
        });

        quote! {
            enum #name {
                #( #variants ),*
            }

            #( #from_impls )*
        }
    }

    pub fn enums(encoding: Encoding) -> impl ToTokens {
        let enums = Group::combinations()
            .into_iter()
            .map(|groups| Self::r#enum(encoding, groups));
        quote! { #( #enums )* }
    }
}

fn code(ch: u8) -> Vec<Symbol> {
    LETTERS
        .iter()
        .find(|(c, _)| *c == ch)
        .unwrap()
        .1
        .iter()
        .map(|c| match c {
            b'.' => Symbol::DOT,
            b'-' => Symbol::LINE,
            _ => unreachable!(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::{GroupData, Item};

    #[test]
    fn check_collisions() {
        let items: Vec<_> = Item::letters()
            .into_iter()
            .chain(Item::prosigns())
            .collect();

        for (i, a) in items.iter().enumerate() {
            for b in &items[..i] {
                let (GroupData::Letter {
                    symbols: a_symbols, ..
                }
                | GroupData::Prosign {
                    symbols: a_symbols, ..
                }) = &a.group
                else {
                    unreachable!()
                };
                let (GroupData::Letter {
                    symbols: b_symbols, ..
                }
                | GroupData::Prosign {
                    symbols: b_symbols, ..
                }) = &b.group
                else {
                    unreachable!()
                };

                assert_ne!(
                    a_symbols, b_symbols,
                    "Collision between {} and {}",
                    a.name, b.name
                )
            }
        }
    }
}
