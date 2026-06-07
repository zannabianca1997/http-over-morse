//! Reader in a buffer

use core::{cmp::min, mem::MaybeUninit, slice};

use crate::{
    Buffer, ReadRegion,
    sync::{DefaultSynchronization, MaybeCriticalSection, NoCriticalSection, Synchronization},
};

/// Readable view into a buffer
///
/// For this to be safe to interact with, only a single [`Reader`] must exist at
/// the same time
pub struct Reader<
    'b,
    T,
    const CAP: usize,
    Sync: Synchronization = DefaultSynchronization,
    CS = NoCriticalSection,
> {
    buffer: &'b Buffer<T, CAP, Sync>,
    cs: CS,
}

impl<'b, T, const CAP: usize, Sync: Synchronization, CS> Reader<'b, T, CAP, Sync, CS>
where
    CS: MaybeCriticalSection<Sync>,
{
    pub(crate) const fn new(buffer: &'b Buffer<T, CAP, Sync>, cs: CS) -> Self {
        Self { buffer, cs }
    }

    /// Return the amount of filled space currently available in the buffer.
    ///
    /// If the writer is writing concurrently with this call, then the amount of
    /// filled space may increase, but it will not decrease below the value
    /// which is returned.
    pub fn available(&self) -> usize {
        let (start, end) = self.fetch_start_and_end();
        end.wrapping_sub(start)
    }

    /// Read a single element
    pub fn pop(&mut self) -> Option<T> {
        // TODO: this is so much simpler that what this chain invokes. Check it
        // is being correctly optimized
        self.read().pop()
    }

    /// Obtain a region to read
    ///
    /// This will return a region where elements can be removed freely from the
    /// buffer. The space will be available to the writer to use when the region
    /// is dropped.
    ///
    /// The returned region will be empty only if the buffer itself is empty.
    pub fn read<'s>(&'s mut self) -> ReadRegion<'s, T, Sync, CS> {
        let (start, len) = self.fetch_start_and_len_contiguous();

        // Pointer to the start of the buffer
        let buffer = self.buffer.content.get().cast::<MaybeUninit<T>>();

        let start = unsafe {
            // Safety: we know the length is CAP, and it's a power of 2, so this is inside the buffer
            buffer.add(start & (CAP - 1))
        };

        let region = unsafe {
            // Safety: we know that start points to a valid element in the
            // buffer. We know that len has been calculated to not wrap around
            // the buffer.
            //
            // This is the only reader, and it is borrowed mutably, so it is
            // safe to create a mutable reference to this part of the buffer as
            // no one else will. The only other creator of mutable views in the
            // buffer is the writer, and creates them only in the uninitialized
            // part.
            slice::from_raw_parts_mut(start, len)
        };

        ReadRegion::new(region, &self.buffer.head, self.cs)
    }

    /// Start and end of the initialized part
    fn fetch_start_and_end(&self) -> (usize, usize) {
        let (head, tail) = self.buffer.get_head_and_tail(&self.cs);

        let start = head;
        let end = tail;

        (start, end)
    }

    /// Start and length of the initialized part, without wrapping around the buffer.
    ///
    /// The returned range will contain _at least_ one element, unless the buffer is empty
    fn fetch_start_and_len_contiguous(&self) -> (usize, usize) {
        let (start, end) = self.fetch_start_and_end();

        // Maximum length without wrapping
        let non_wrapping_len = CAP - (start & (CAP - 1));
        // Filled length
        let init_len = end.wrapping_sub(start);
        // End of the non wrapping area
        let init_non_wrapping_len = min(non_wrapping_len, init_len);

        (start, init_non_wrapping_len)
    }
}
