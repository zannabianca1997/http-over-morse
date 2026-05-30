///! Symbol definition
///
/// Defined here to be shared with `build.rs`

/// Symbol on the wire
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Symbol {
    /// A dot
    DOT,
    /// A line
    LINE,
}
pub use Symbol::*;

impl phf::PhfHash for Symbol {
    fn phf_hash<H: core::hash::Hasher>(&self, state: &mut H) {
        match self {
            DOT => false,
            LINE => true,
        }
        .phf_hash(state);
    }
}
