use std::{collections::BTreeSet, env, fs, path::Path};

use proc_macro2::Literal;
use quote::{format_ident, quote};

/// Iter all u16 primes
struct PrimeIter {
    found: BTreeSet<u16>,
    tested: u16,
}

impl PrimeIter {
    fn new() -> Self {
        Self {
            found: BTreeSet::new(),
            tested: 0,
        }
    }
}

impl Iterator for PrimeIter {
    type Item = u16;

    fn next(&mut self) -> Option<Self::Item> {
        if self.tested == 0 {
            self.tested = 1;
            return Some(2);
        }

        loop {
            self.tested = self.tested.checked_add(2)?;

            if self
                .found
                .iter()
                .copied()
                .take_while(|d| d.checked_pow(2).is_some_and(|p2| p2 < self.tested))
                .any(|d| self.tested % d == 0)
            {
                // not prime
                continue;
            }

            self.found.insert(self.tested);
            break Some(self.tested);
        }
    }
}

fn main() {
    let primes: Vec<_> = PrimeIter::new().collect();

    let enum_arms = primes.iter().map(|p| {
        let name = format_ident!("P{p:0X}");
        let lit = Literal::u16_unsuffixed(*p);
        quote! { #name = #lit }
    });
    let slice_elements = primes.iter().map(|p| {
        let name = format_ident!("P{p:0X}");
        quote! { Prime :: #name }
    });
    let count = Literal::usize_unsuffixed(primes.len());

    let out = quote! {
        /// An u16 prime
        #[derive(Clone, Copy)]
        #[repr(u16)]
        pub enum Prime {
            #( #enum_arms ),*
        }

        /// Count of all primes
        const COUNT: usize = #count;

        /// All primes in sorted order
        static ALL: [Prime; COUNT] = [ #( #slice_elements ),* ];
    };

    let outfile = Path::new(&env::var_os("OUT_DIR").unwrap()).join("generated.rs");

    fs::write(&outfile, out.to_string()).unwrap();

    cargo_emit::rustc_env!("GENERATED_RS", "{}", outfile.display())
}
