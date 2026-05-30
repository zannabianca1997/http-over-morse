//! Strategies to convert bytes to morse letters

mod sealed {
    pub trait Sealed {}
}

/// Strategy
pub trait Strategy: sealed::Sealed {
    type ReadData: Sized;
    fn new_read() -> Self::ReadData;

    type WriteData: Sized;
    fn new_write() -> Self::WriteData;
}

/// Discard all letters that are not encodable
pub enum Discard {}
impl sealed::Sealed for Discard {}
impl Strategy for Discard {
    type ReadData = ();
    fn new_read() -> Self::ReadData {}
    type WriteData = ();
    fn new_write() -> Self::WriteData {}
}
