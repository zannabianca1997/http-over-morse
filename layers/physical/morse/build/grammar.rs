use std::{collections::BTreeMap, fs};

use anyhow::{Context, Error, Result, anyhow, bail};
use itertools::Itertools;
use serde::Deserialize;

use crate::src::symbol::Symbol;

#[derive(Debug, Deserialize)]
pub struct Grammar {
    pub letters: Letters,
}

impl Grammar {
    const PATH: &str = "Grammar.toml";

    pub fn load() -> Result<Self> {
        cargo_emit::rerun_if_changed!(Self::PATH);
        let file_content = fs::read(Self::PATH).context("Cannot read grammar file")?;
        toml::from_slice(&file_content).context("Invalid grammar file")
    }
}

#[derive(Debug, Deserialize)]
#[serde(try_from = "BTreeMap<String, String>")]
pub struct Letters {
    defs: BTreeMap<u8, Vec<Symbol>>,
}

impl TryFrom<BTreeMap<String, String>> for Letters {
    type Error = Error;

    fn try_from(value: BTreeMap<String, String>) -> Result<Self> {
        let mut defs = BTreeMap::new();

        for (ch, symbols) in value {
            let ch = ch
                .chars()
                .exactly_one()
                .map_err(|err| anyhow!("String has more than one char: {err}"))
                .context("Letters should be made of a single char")?;
            let ch = u8::try_from(ch).context("Letters should be valid ascii")?;

            if ch != ch.to_ascii_uppercase() {
                cargo_emit::warning!(
                    "Letters are automatically made uppercase, `{}` won't be matched",
                    ch as char
                )
            }

            let symbols = symbols
                .chars()
                .map(|s| {
                    Ok(match s {
                        '-' => Symbol::LINE,
                        '.' => Symbol::DOT,
                        _ => bail!("Invalid symbol {s}"),
                    })
                })
                .try_collect()?;

            defs.insert(ch, symbols);
        }

        Ok(Self { defs })
    }
}

impl IntoIterator for Letters {
    type Item = (u8, Vec<Symbol>);

    type IntoIter = <BTreeMap<u8, Vec<Symbol>> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.defs.into_iter()
    }
}
