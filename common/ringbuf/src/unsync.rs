//! Direct access to the buffer through mutable borrowing reader and writer
//!
//! Avoid paying the cost of synchronization

use crate::{Index, sealed::Sealed};

pub use reader::Reader;
pub use region::{ReadRegion, WriteRegion, WriteRegionSlot};
pub use writer::Writer;

mod reader;
mod region;
mod writer;

impl Sealed for usize {}
impl Index for usize {
    const ZERO: Self = 0;

    fn get_mut(&mut self) -> &mut usize {
        self
    }

    unsafe fn get_unsync(&self) -> usize {
        *self
    }
}

pub type Buffer<T, const CAP: usize, Idx = usize> = crate::Buffer<T, CAP, Idx>;

impl<T, const CAP: usize, Idx: Index> crate::Buffer<T, CAP, Idx> {
    /// Get a writing handle to this buffer through a mutable reference
    pub const fn writer_mut(&mut self) -> Writer<'_, T, CAP, Idx> {
        Writer::new(self)
    }

    /// Get a reading handle to this buffer through a mutable reference
    pub const fn reader_mut(&mut self) -> Reader<'_, T, CAP, Idx> {
        Reader::new(self)
    }

    fn get_head_and_tail_unsync(&self) -> (usize, usize) {
        // Safety: callers (the unsync Reader/Writer) hold this buffer through
        // a mutable borrow, guaranteeing exclusive access.
        unsafe { (Index::get_unsync(&self.head), Index::get_unsync(&self.tail)) }
    }
}
