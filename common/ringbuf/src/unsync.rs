//! Direct access to the buffer through mutable borrowing reader and writer
//!
//! Avoid paying the cost of synchronization

use crate::{ConstZero, ProvidesIndices, sealed::Sealed};

pub use reader::Reader;
pub use region::{ReadRegion, WriteRegion, WriteRegionSlot};
pub use writer::Writer;

mod reader;
mod region;
mod writer;

/// Buffer that can be accessed only through the unsync API
///
/// This avoid setting up any syncronization ballet
pub enum Unsync {}
impl Sealed for Unsync { }
impl ProvidesIndices for Unsync {
    type Usize = usize;

    fn get_mut(var: &mut Self::Usize) -> &mut usize {
        var
    }

    unsafe fn get(var: &Self::Usize) -> usize {
        *var
    }
}
impl ConstZero for usize {
    const ZERO: Self = 0;
}

pub type Buffer<T, const CAP: usize, Sync = Unsync> = crate::Buffer<T, CAP, Sync>;

impl<T, const CAP: usize, Sync: ProvidesIndices> crate::Buffer<T, CAP, Sync> {
    /// Get a writing handle to this buffer through a mutable reference
    pub const fn writer_mut(&mut self) -> Writer<'_, T, CAP, Sync> {
        Writer::new(self)
    }

    /// Get a reading handle to this buffer through a mutable reference
    pub const fn reader_mut(&mut self) -> Reader<'_, T, CAP, Sync> {
        Reader::new(self)
    }

    fn get_head_and_tail_unsync(&self) -> (usize, usize) {
        // Safety: callers (the unsync Reader/Writer) hold this buffer through
        // a mutable borrow, guaranteeing exclusive access.
        unsafe { (Sync::get(&self.head), Sync::get(&self.tail)) }
    }
}
