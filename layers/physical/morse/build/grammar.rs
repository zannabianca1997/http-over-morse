use std::{
    borrow::Cow,
    collections::{BTreeMap, HashMap},
    fs,
    num::{NonZero, NonZeroU8},
};

use anyhow::{Context, Error, Result, anyhow, bail};
use itertools::Itertools;
use schemars::{JsonSchema, Schema, SchemaGenerator, json_schema};
use serde::Deserialize;

use crate::src::symbol::Symbol;

/// The morse code definition
#[derive(Debug, Deserialize, JsonSchema)]
pub struct Grammar {
    /// Letters we understand
    #[serde(default)]
    pub letters: Letters,

    /// Prosigns made of those letters
    #[serde(default)]
    pub prosigns: HashMap<String, Prosign>,
}

impl Grammar {
    const PATH: &str = "Grammar.toml";

    pub fn load() -> Result<Self> {
        cargo_emit::rerun_if_changed!(Self::PATH);
        let file_content = fs::read(Self::PATH).context("Cannot read grammar file")?;
        toml::from_slice(&file_content).context("Invalid grammar file")
    }
}

#[derive(Debug, Deserialize, Default)]
#[serde(try_from = "BTreeMap<String, String>")]
pub struct Letters(BTreeMap<NonZeroU8, Vec<Symbol>>);

impl JsonSchema for Letters {
    fn schema_name() -> Cow<'static, str> {
        Cow::Borrowed("Letters")
    }

    fn schema_id() -> Cow<'static, str> {
        Cow::Borrowed(concat!(module_path!(), "::Letters"))
    }

    fn json_schema(_gen: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "description": "A map from single ASCII characters to their Morse code symbol sequences. Keys are automatically uppercased; lowercase keys will not be matched.",
            "type": "object",
            "additionalProperties": {
                "type": "string",
                "description": "A non-empty string composed exclusively of '.' (dot) and '-' (dash/line) characters.",
                "pattern": "^[.\\-]+$",
                "minLength": 1
            },
            "propertyNames": {
                "description": "A single ASCII character (will be treated as uppercase).",
                "type": "string",
                "minLength": 1,
                "maxLength": 1,
                "pattern": "^[\\u0000-\\u007F]$"
            }
        })
    }
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

            if ch.is_ascii_whitespace() {
                bail!("Whitespace is reserved for internal use")
            }

            let ch = NonZero::new(ch).expect("\\0 is reserved for internal use");

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

        Ok(Self(defs))
    }
}

impl IntoIterator for Letters {
    type Item = (NonZero<u8>, Vec<Symbol>);

    type IntoIter = <BTreeMap<NonZero<u8>, Vec<Symbol>> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

/// A prosign: a series of letters without a space between.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct Prosign {
    /// Letters that make up this prosign
    #[schemars(pattern(r#"^[A-Z0-9]+$"#), length(min = 1))]
    pub code: String,

    /// Character this prosign is mapped to
    #[schemars(range(max = 127))]
    pub maps_to: NonZeroU8,
}
