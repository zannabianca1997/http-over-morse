use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use quote::ToTokens;
use serde::{Deserialize, Serialize};

use super::{
    Found,
    std::{
        collections::BTreeMap,
        fs::{File, OpenOptions},
        io::{self, Seek, SeekFrom},
        path::Path,
        string::String,
        string::ToString,
        time::Instant,
        vec::Vec,
    },
};

#[derive(Debug)]
pub struct Cache {
    file: File,
    dirty: bool,
    content: Content,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Content {
    updated: DateTime<Utc>,
    total_time: Duration,
    tables: Vec<Entry>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Entry {
    created: DateTime<Utc>,
    calc_time: Duration,
    mapping: BTreeMap<u8, serde_json::Value>,
    typ: String,
    def: String,
    size: usize,
}

impl Entry {
    fn new(
        Found {
            mapping,
            typ,
            def,
            size,
        }: Found,
        calc_time: Duration,
    ) -> Self {
        Self {
            calc_time,
            created: Utc::now(),
            mapping,
            typ: typ.to_token_stream().to_string(),
            def: def.to_token_stream().to_string(),
            size,
        }
    }
    fn parse(&self) -> Found {
        let Self {
            mapping,
            typ,
            def,
            size,
            ..
        } = self;
        Found {
            mapping: mapping.clone(),
            typ: syn::parse_str(typ).unwrap(),
            def: syn::parse_str(&def).unwrap(),
            size: *size,
        }
    }
}

impl Cache {
    /// Load the cache file
    pub fn load(path: &Path) -> Result<Self> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)
            .or_else(|err| {
                if err.kind() == io::ErrorKind::NotFound {
                    // Init new cache file
                    let content = Content {
                        updated: Utc::now(),
                        total_time: Duration::zero(),
                        tables: Vec::new(),
                    };

                    let mut file = OpenOptions::new()
                        .create_new(true)
                        .read(true)
                        .write(true)
                        .open(path)
                        .context("Cannot create cache file")?;

                    serde_json::to_writer_pretty(&mut file, &content)
                        .context("Cannot write cache file")?;

                    file.seek(SeekFrom::Start(0))?;

                    Ok(file)
                } else {
                    Err(err).context("Cannot open cache file")
                }
            })?;

        let content = serde_json::from_reader(&mut file).context("Cannot parse cache file")?;

        Ok(Self {
            file,
            content,
            dirty: false,
        })
    }

    /// Save the cache file
    pub fn save(&mut self) -> Result<()> {
        if !self.dirty {
            return Ok(());
        }

        self.file.seek(SeekFrom::Start(0))?;

        serde_json::to_writer_pretty(&mut self.file, &self.content)
            .context("Cannot write cache file")?;

        let len = self.file.stream_position()?;
        self.file.set_len(len)?;

        self.dirty = false;

        Ok(())
    }

    /// Get a cached table
    pub fn get<T: Serialize>(&self, fun: impl Fn(u8) -> Option<T>) -> Option<Found> {
        let mapping = (u8::MIN..=u8::MAX)
            .filter_map(|i| fun(i).map(|v| (i, serde_json::to_value(v).unwrap())))
            .collect();

        self.content
            .tables
            .iter()
            .find(|t| t.mapping == mapping)
            .map(Entry::parse)
    }

    /// Get or calculate a table
    pub fn get_or_calculate<T: Eq + Ord + ToTokens + Send + Serialize>(
        &mut self,
        typ_name: impl ToTokens,
        fun: impl Fn(u8) -> Option<T> + Send + Sync,
    ) -> Found {
        if let Some(cached) = self.get(&fun) {
            return cached;
        }

        let start = Instant::now();
        let found = super::search(typ_name, fun);
        let elapsed = Duration::from_std(start.elapsed()).unwrap();

        self.content.tables.push(Entry::new(found.clone(), elapsed));

        self.dirty = true;
        self.content.updated = Utc::now();
        self.content.total_time += elapsed;

        found
    }
}

impl Drop for Cache {
    fn drop(&mut self) {
        self.save().expect("Error in saving cache")
    }
}
