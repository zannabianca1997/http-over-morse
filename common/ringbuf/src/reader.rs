//! Reader in a buffer

use core::{cmp::min, mem::MaybeUninit, slice};

use crate::{
    Buffer, ReadRegion,
    sync::{DefaultSynchronization, Synchronization},
};

/// Readable view into a buffer
///
/// For this to be safe to interact with, only a single [`Reader`] must exist at
/// the same time
pub struct Reader<
    'b,
    T,
    const CAP: usize,
    BufferSync: Synchronization = DefaultSynchronization,
    Sync: Synchronization<Usize = BufferSync::Usize> = BufferSync,
> {
    buffer: &'b Buffer<T, CAP, BufferSync>,
    sync: Sync,
}

impl<
    'b,
    T,
    const CAP: usize,
    BufferSync: Synchronization,
    Sync: Synchronization<Usize = BufferSync::Usize>,
> Reader<'b, T, CAP, BufferSync, Sync>
{
    pub(crate) const fn new(buffer: &'b Buffer<T, CAP, BufferSync>, sync: Sync) -> Self {
        Self { buffer, sync }
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
        self.read(|r| r.pop())
    }

    /// Obtain a region to read
    ///
    /// This will return a region where elements can be removed freely from the
    /// buffer. The space will be available to the writer to use when the region
    /// is dropped.
    ///
    /// The returned region will be empty only if the buffer itself is empty.
    pub fn read<R>(&mut self, fun: impl FnOnce(&mut ReadRegion<'_, T, Sync>) -> R) -> R {
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

        let mut region = ReadRegion::new(region, &self.buffer.head, self.sync);

        let res = fun(&mut region);

        // Drop the region, updating the buffer
        drop(region);

        res
    }

    /// Start and end of the initialized part
    fn fetch_start_and_end(&self) -> (usize, usize) {
        let (head, tail) = self.buffer.get_head_and_tail(self.sync);

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

#[cfg(test)]
#[cfg(target_has_atomic = "ptr")]
mod tests {
    use crate::{Buffer, sync::Atomics};

    /// A read region is empty exactly when the buffer is empty.
    #[test]
    fn read_region_empty_only_when_empty() {
        let mut buf = Buffer::<i32, 4, Atomics>::new();
        let (mut w, mut r) = buf.split();

        assert!(r.read(|reg| reg.is_empty()));
        w.push(7).unwrap();
        assert!(!r.read(|reg| reg.is_empty()));
    }

    /// When the filled data wraps the backing array, a single read region is
    /// truncated at the wrap boundary (it does not span the seam).
    #[test]
    fn read_region_truncates_at_wrap() {
        let mut buf = Buffer::<i32, 4, Atomics>::new();
        let (mut w, mut r) = buf.split();

        for i in 0..4 {
            w.push(i).unwrap();
        }
        assert_eq!(r.pop(), Some(0));
        assert_eq!(r.pop(), Some(1));
        // Re-fill the freed slots, so logical data now wraps: slots 2,3,0,1.
        w.push(4).unwrap();
        w.push(5).unwrap();

        // head is at index 2; the contiguous run is slots 2,3 -> length 2.
        assert_eq!(r.read(|reg| reg.len()), 2);

        // Draining still yields the correct order across the seam.
        assert_eq!(r.pop(), Some(2));
        assert_eq!(r.pop(), Some(3));
        assert_eq!(r.pop(), Some(4));
        assert_eq!(r.pop(), Some(5));
        assert_eq!(r.pop(), None);
    }
}
