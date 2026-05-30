//! Strategies to convert bytes to morse letters

use crate::encoding::LetterOrSpace;

mod sealed {
    pub trait Sealed {}
}

pub trait Strategy: sealed::Sealed {
    type ReadData: Sized;
    fn init_read() -> Self::ReadData;
}

pub enum Discard {}
impl sealed::Sealed for Discard {}
impl Strategy for Discard {
    type ReadData = ();
    fn init_read() -> Self::ReadData {}
}

pub struct Base32;
impl sealed::Sealed for Base32 {}
impl Strategy for Base32 {
    type ReadData = B32ReadData;
    fn init_read() -> Self::ReadData {
        B32ReadData {
            len: 0,
            buf: [None; 5],
        }
    }
}

pub struct B32ReadData {
    pub len: u8,
    pub buf: [Option<LetterOrSpace>; 5],
}
