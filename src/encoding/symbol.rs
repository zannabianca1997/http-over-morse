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

/// Mixes seven bytes and a symbol together
pub(crate) const fn u7_and_symbol(u7: u8, symbol: Symbol) -> u8 {
    (u7 << 1)
        | match symbol {
            DOT => 0,
            LINE => 1,
        }
}
