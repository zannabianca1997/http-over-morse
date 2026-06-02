use std::collections::BTreeMap;
use std::env;
use std::hash::Hash;
use std::iter;

use anyhow::{Result, bail};
use itertools::Itertools;
use rayon::prelude::*;
use serde::Serialize;

use crate::build::cache::TableCacheInfo;
use crate::build::primes::PRIMES;
use crate::src::perfect_hash::Hasher;

pub fn calculate_perfect_hash<T: Clone + Eq + Serialize + Hash + Sync>(
    values: BTreeMap<u8, T>,
) -> Result<(Hasher, Box<[Option<T>]>)> {
    cargo_emit::rerun_if_env_changed!("CALC_HASHER");

    let calc_hash = env::var_os("CALC_HASHER").is_some();

    let mut cache_info = TableCacheInfo::load()?;

    if !calc_hash {
        if let Some(hasher) = cache_info.get(&values)? {
            let mut table_data: Box<[Option<T>]> = iter::from_fn(|| Some(None))
                .take(hasher.size.get() as usize)
                .collect();
            for (key, value) in values {
                table_data[hasher.hash(key) as usize] = Some(value);
            }
            return Ok((hasher, table_data));
        } else {
            cargo_emit::warning!("Recalculating perfect hasher - `.phcache` entry missing")
        }
    } else {
        cargo_emit::warning!("Recalculating perfect hasher - CALC_HASHER set")
    }

    let min_size = values.values().unique().count();
    if min_size >= 256 {
        bail!(
            "Refusing to create table of more than 256 elements as the identity would be more efficient"
        );
    }

    let values: Vec<(u8, T)> = values.into_iter().collect();

    let hasher = (min_size..=256).find_map(|size| {
        (1..=u16::MAX)
            .into_par_iter()
            .flat_map(|multiplier| {
                PRIMES.par_iter().rev().copied().map(move |modulus| Hasher {
                    multiplier,
                    modulus: std::num::NonZeroU16::new(modulus).unwrap(),
                    size: std::num::NonZeroU16::new(size as u16).unwrap(),
                })
            })
            .find_any(|hasher| {
                let mut mapped_values: [Option<&T>; 256] = [None; 256];

                for (key, value) in &values {
                    let mapped = hasher.hash(*key);

                    if mapped_values[mapped as usize].is_some_and(|mapped| mapped != value) {
                        return false;
                    }
                    mapped_values[mapped as usize] = Some(value);
                }

                true
            })
    });

    let Some(hasher) = hasher else {
        bail!(
            "No hasher found in search space (this should be impossible if the hashing function contains the identity"
        );
    };

    let values = values.into_iter().collect();

    cache_info.set(&values, hasher)?;
    cache_info.save()?;

    let mut table_data: Box<[Option<T>]> = iter::from_fn(|| Some(None))
        .take(hasher.size.get() as usize)
        .collect();

    for (key, value) in values {
        table_data[hasher.hash(key) as usize] = Some(value);
    }

    Ok((hasher, table_data))
}
