use core::{cmp::min, mem::MaybeUninit, slice};

use crate::{
    Buffer, WriteRegion,
    sync::{DefaultSynchronization, MaybeCriticalSection, NoCriticalSection, Synchronization},
};

/// Writable view into a buffer
///
/// For this to be safe to interact with, only a single [`Writer`] must exist at
/// the same time
pub struct Writer<
    'b,
    T,
    const CAP: usize,
    Sync: Synchronization = DefaultSynchronization,
    CS = NoCriticalSection,
> {
    buffer: &'b Buffer<T, CAP, Sync>,
    cs: CS,
}

impl<'b, T, const CAP: usize, Sync: Synchronization, CS> Writer<'b, T, CAP, Sync, CS>
where
    CS: MaybeCriticalSection<Sync>,
{
    pub(crate) const fn new(buffer: &'b Buffer<T, CAP, Sync>, cs: CS) -> Self {
        Self { buffer, cs }
    }

    /// Return the amount of empty space currently available in the buffer.
    ///
    /// If the reader is reading concurrently with this call, then the amount of
    /// empty space may increase, but it will not decrease below the value which
    /// is returned.
    pub fn available(&self) -> usize {
        let (start, end) = self.fetch_start_and_end();
        end.wrapping_sub(start)
    }

    /// Write a single element
    pub fn push(&mut self, item: T) -> Result<(), T> {
        // TODO: this is so much simpler that what this chain invokes. Check it
        // is being correctly optimized
        self.write(|r| r.push(item).map(|_| ()))
    }

    /// Obtain a region to write
    ///
    /// This will return a region where elements can be added freely into the
    /// buffer. The space will be available to the reader to read when the
    /// region is dropped.
    ///
    /// The returned region will be empty only if the buffer is full.
    pub fn write<R>(&mut self, fun: impl FnOnce(&mut WriteRegion<'_, T, Sync, CS>) -> R) -> R {
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
            // This is the only writer, and it is borrowed mutably, so it is
            // safe to create a mutable reference to this part of the buffer as
            // no one else will. The only other creator of mutable views in the
            // buffer is the reader, and creates them only in the initialized
            // part.
            slice::from_raw_parts_mut(start, len)
        };

        let mut region = WriteRegion::new(region, &self.buffer.tail, self.cs);

        let res = fun(&mut region);

        // Drop the region, updating the buffer
        drop(region);

        res
    }

    /// Start and end of the uninitialized part
    fn fetch_start_and_end(&self) -> (usize, usize) {
        let (head, tail) = self.buffer.get_head_and_tail(&self.cs);

        let start = tail;
        let end = head.wrapping_add(CAP);

        (start, end)
    }

    /// Start and length of the uninitialized part, without wrapping around the buffer.
    ///
    /// The returned range will contain _at least_ one element, unless the
    /// buffer is full
    fn fetch_start_and_len_contiguous(&self) -> (usize, usize) {
        let (start, end) = self.fetch_start_and_end();

        // Maximum length without wrapping
        let non_wrapping_len = CAP - (start & (CAP - 1));
        // Filled length
        let uninit_len = end.wrapping_sub(start);
        // End of the non wrapping area
        let uninit_non_wrapping_len = min(non_wrapping_len, uninit_len);

        (start, uninit_non_wrapping_len)
    }
}

#[cfg(test)]
#[cfg(target_has_atomic = "ptr")]
mod tests {
    use crate::{Buffer, sync::Atomics};

    /// A write region is empty exactly when the buffer is full.
    #[test]
    fn write_region_empty_only_when_full() {
        let mut buf = Buffer::<i32, 4, Atomics>::new();
        let (mut w, _r) = buf.split();

        assert!(!w.write(|reg| reg.is_empty()));
        for i in 0..4 {
            w.push(i).unwrap();
        }
        assert!(w.write(|reg| reg.is_empty()));
    }

    /// When the free space wraps the backing array, a single write region is
    /// truncated at the wrap boundary.
    #[test]
    fn write_region_truncates_at_wrap() {
        let mut buf = Buffer::<i32, 4, Atomics>::new();
        let (mut w, mut r) = buf.split();

        for i in 0..3 {
            w.push(i).unwrap();
        }
        assert_eq!(r.pop(), Some(0)); // free space is now slots 3 then 0

        // tail is at index 3; the contiguous free run is just slot 3 -> len 1.
        assert_eq!(w.write(|reg| reg.len()), 1);
        assert_eq!(w.available(), 2);
    }
}
