///! Symbol definition
///
/// Defined here to be shared with `build.rs`

/// Symbol on the wire
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Symbol {
    /// A dot
    DOT = 0b0000_0000,
    /// A line
    LINE = 0b1000_0000,
}
pub use Symbol::*;

/// Mixes seven bytes and a symbol together
pub(crate) const fn u7_and_symbol(u7: u8, symbol: Symbol) -> u8 {
    u7 | (symbol as u8)
}
