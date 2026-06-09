//! Direct access to the buffer through mutable borrowing reader and writer
//!
//! Avoid paying the cost of synchronization

use crate::{ConstZero, ProvidesIndices};

pub use reader::Reader;
pub use region::{ReadRegion, WriteRegion, WriteRegionSlot};
pub use writer::Writer;

mod reader;
mod region;
mod writer;

pub enum Unsync {}
impl ProvidesIndices for Unsync {
    type Usize = usize;

    fn get_mut(var: &mut Self::Usize) -> &mut usize {
        var
    }
}
impl ConstZero for usize {
    const ZERO: Self = 0;
}

pub type Buffer<T, const CAP: usize, Sync = Unsync> = crate::Buffer<T, CAP, Sync>;

impl<T, const CAP: usize, Sync: ProvidesIndices> crate::Buffer<T, CAP, Sync> {
    /// Get a writing handle to this buffer through a mutable reference
    pub const fn writer_mut(&mut self) -> Writer<'_, T, CAP> {
        Writer::new(self)
    }

    /// Get a reading handle to this buffer through a mutable reference
    pub const unsafe fn reader_mut(&mut self) -> Reader<'_, T, CAP> {
        Reader::new(self)
    }

    fn get_head_and_tail_mut(&mut self) -> (usize, usize) {
        let head = *Sync::get_mut(&mut self.head);
        let tail = *Sync::get_mut(&mut self.tail);

        (head, tail)
    }
}
