use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use quote::ToTokens;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{
    Found,
    std::{
        collections::BTreeMap,
        fs::{File, OpenOptions},
        io::{self, Seek, SeekFrom},
        path::Path,
        string::String,
        string::ToString,
        sync::Mutex,
        time::Instant,
        vec::Vec,
    },
};

#[derive(Debug)]
pub struct Cache {
    file: File,
    content: Mutex<Content>,
}

fn default_dirty() -> bool {
    false
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Content {
    #[serde(skip, default = "default_dirty")]
    dirty: bool,
    updated: DateTime<Utc>,
    total_time: iso8601_duration::Duration,
    tables: Vec<Entry>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Entry {
    created: DateTime<Utc>,
    calc_time: iso8601_duration::Duration,
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
            calc_time: iso8601_duration::Duration::parse(&calc_time.to_string()).unwrap(),
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
                        dirty: false,
                        updated: Utc::now(),
                        total_time: iso8601_duration::Duration::new(0., 0., 0., 0., 0., 0.01),
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

        Ok(Self { file, content })
    }

    /// Save the cache file
    pub fn save(&mut self) -> Result<()> {
        if !self.content.get_mut().unwrap().dirty {
            return Ok(());
        }

        self.file.seek(SeekFrom::Start(0))?;

        serde_json::to_writer_pretty(&mut self.file, &self.content)
            .context("Cannot write cache file")?;

        let len = self.file.stream_position()?;
        self.file.set_len(len)?;

        self.content.get_mut().unwrap().dirty = false;

        Ok(())
    }

    /// Get a cached table
    fn get_inner<T: Serialize>(
        &self,
        fun: impl Fn(u8) -> Option<T>,
    ) -> (Option<Found>, BTreeMap<u8, Value>) {
        let mapping = (u8::MIN..=u8::MAX)
            .filter_map(|i| fun(i).map(|v| (i, serde_json::to_value(v).unwrap())))
            .collect();

        let found = self
            .content
            .lock()
            .unwrap()
            .tables
            .iter()
            .find(|t| t.mapping == mapping)
            .map(Entry::parse);

        (found, mapping)
    }

    /// Get a cached table
    pub fn get<T: Serialize>(&self, fun: impl Fn(u8) -> Option<T>) -> Option<Found> {
        self.get_inner(fun).0
    }

    /// Get or calculate a table
    pub fn get_or_calculate<T: Eq + Ord + ToTokens + Send + Serialize + Clone + Sync>(
        &self,
        typ_name: impl ToTokens,
        fun: impl Fn(u8) -> Option<T> + Send + Sync,
    ) -> Found {
        let (cached, mapping) = self.get_inner(&fun);
        if let Some(cached) = cached {
            return cached;
        }

        let start = Instant::now();
        let found = super::search(typ_name, fun);
        let elapsed = Duration::from_std(start.elapsed()).unwrap();

        let mut content = self.content.lock().unwrap();

        if let Some(cached) = content
            .tables
            .iter()
            .find(|t| t.mapping == mapping)
            .map(Entry::parse)
        {
            // Someone got it before, we calculated it twice
            return cached;
        }

        content.tables.push(Entry::new(found.clone(), elapsed));

        content.dirty = true;
        content.updated = Utc::now();
        let time_delta = content.total_time.to_chrono().unwrap() + elapsed;
        content.total_time = iso8601_duration::Duration::parse(&time_delta.to_string()).unwrap();

        found
    }
}

impl Drop for Cache {
    fn drop(&mut self) {
        self.save().expect("Error in saving cache")
    }
}
