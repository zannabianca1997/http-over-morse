//! Panic configuration
//!
//! On debug, setup so the panic halts the avr. On release instead make so
//! panics call an unlinked function, effectively prohibiting panics as a whole

#[cfg(debug_assertions)]
use panic_halt as _;

#[cfg(not(debug_assertions))]
use panic_never as _;
