use std::cmp::{Reverse, min};
use std::range::Range;

use itertools::Itertools;

pub fn supersequence<K, T: Eq>(
    definitions: impl IntoIterator<Item = (K, Vec<T>)>,
) -> (Vec<T>, Vec<(K, Range<usize>)>) {
    let mut supersequence = vec![];
    let mut ranges = vec![];

    for (ch, mut symbols) in definitions
        .into_iter()
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
            supersequence.extend(symbols.drain(overlap..));
            let end = supersequence.len();

            ranges.push((ch, (start..end).into()));
        }
    }

    (supersequence, ranges)
}
