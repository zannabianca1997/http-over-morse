//! Contiguous views into buffers

use core::{marker::PhantomData, mem::MaybeUninit, ptr::NonNull};

use crate::sync::{DefaultSynchronization, Synchronization};

/// A contiguous view in a buffer
struct RawRegion<'o, Sync: Synchronization = DefaultSynchronization> {
    /// Slice of the buffer this region has unique use of
    ///
    /// Stored as a type erased pointer to select variance after
    region: NonNull<()>,
    /// Lenght of the region
    len: usize,
    /// Length of the region at its creation
    ///
    /// The region only ever shrinks from the front
    original_len: usize,
    /// Head or tail of the buffer, the one set to the start of the region
    start_index_storage: &'o Sync::Usize,
    /// Syncronization primitive used
    sync: Sync,
}

impl<'o, Sync: Synchronization> RawRegion<'o, Sync> {
    fn new<T>(
        region: &'o mut [MaybeUninit<T>],
        start_index_storage: &'o Sync::Usize,
        sync: Sync,
    ) -> Self {
        let len = region.len();
        let region = NonNull::from_mut(region).cast();
        Self {
            original_len: len,
            region,
            len,
            start_index_storage,
            sync,
        }
    }

    fn len(&self) -> usize {
        self.len
    }

    fn is_empty(&self) -> bool {
        self.len == 0
    }
}

/// Here is where the magic happens
///
/// Both push and pop do not touch the buffer, only update the region bounds. On
/// `drop` the region updates the buffer head or tail.
///
/// If the user leaks the region there is no UB, as the owner would remain
/// borrowed mutably forever, and only one can exist.
//
// There is a possibility of UB tho:
// - Create a `ReadRegion`
// - Read a value
// - Wrap it in ManuallyDrop and throw it away without running drop
// - Create another region
// - Read the same value
// - duplicate values!
//
// Avoiding this looks impossible without runtime overhead... so we just never
// give the caller an owned [`Region`]
impl<'o, Sync: Synchronization> Drop for RawRegion<'o, Sync> {
    fn drop(&mut self) {
        // The region only shrinks from the front, so the number of slots read
        // or written is the difference between the original and current length.
        let used = self.original_len - self.len;
        // Update the start of the region in the buffer
        self.sync.wrapping_add(self.start_index_storage, used)
    }
}

pub struct ReadRegion<'o, T, Sync: Synchronization = DefaultSynchronization> {
    raw: RawRegion<'o, Sync>,
    _phantom: PhantomData<T>,
}

impl<'o, T, Sync: Synchronization> ReadRegion<'o, T, Sync> {
    pub(in crate::sync) fn new(
        region: &'o mut [MaybeUninit<T>],
        start_index_storage: &'o Sync::Usize,
        sync: Sync,
    ) -> Self {
        Self {
            raw: RawRegion::new(region, start_index_storage, sync),
            _phantom: PhantomData,
        }
    }

    /// Remove an element from the region
    pub fn pop(&mut self) -> Option<T> {
        // Take out a value, updating the slice
        if self.raw.len() == 0 {
            return None;
        }

        let region: NonNull<T> = self.raw.region.cast();

        self.raw.len -= 1;
        self.raw.region = unsafe {
            // Safety: len ensures the memory is from the right allocation
            region.add(1)
        }
        .cast();

        Some(unsafe {
            // Safety: ReadRegion can only be created by a reader, that ensures
            // it is pointing to fully initialized memory. The type is covariant
            // in `T`, meaning `T` might be a supertype now -> safe to return a
            // subtype
            region.read()
        })
    }

    /// Current length of the region
    pub fn len(&self) -> usize {
        self.raw.len()
    }

    /// Is the region empty
    pub fn is_empty(&self) -> bool {
        self.raw.is_empty()
    }

    /// Get a view to the region elements
    pub fn as_slice(&self) -> &[T] {
        unsafe {
            // Safety: ReadRegion can only be created by a reader, that ensures
            // it is pointing to fully initialized memory. The type is covariant
            // in `T`, meaning `T` might be a supertype now -> safe to return a
            // subtype
            NonNull::slice_from_raw_parts(self.raw.region.cast(), self.raw.len).as_ref()
        }
    }
}

pub struct WriteRegion<'o, T, Sync: Synchronization = DefaultSynchronization> {
    raw: RawRegion<'o, Sync>,
    _phantom: PhantomData<fn(T)>,
}

impl<'o, T, Sync: Synchronization> WriteRegion<'o, T, Sync> {
    pub(in crate::sync) fn new(
        region: &'o mut [MaybeUninit<T>],
        start_index_storage: &'o Sync::Usize,
        sync: Sync,
    ) -> Self {
        Self {
            raw: RawRegion::new(region, start_index_storage, sync),
            _phantom: PhantomData,
        }
    }

    /// Get a slot in the region
    pub fn slot(&mut self) -> Option<WriteRegionSlot<'_, T>> {
        if self.raw.is_empty() {
            return None;
        } else {
            return Some(WriteRegionSlot(
                &mut self.raw.region,
                &mut self.raw.len,
                PhantomData,
            ));
        }
    }

    /// Add an element into the region
    pub fn push(&mut self, item: T) -> Result<(), T> {
        match self.slot() {
            Some(slot) => Ok(slot.fill(item)),
            None => Err(item),
        }
    }

    pub fn len(&self) -> usize {
        self.raw.len()
    }

    pub fn is_empty(&self) -> bool {
        self.raw.is_empty()
    }
}

pub struct WriteRegionSlot<'o, T>(&'o mut NonNull<()>, &'o mut usize, PhantomData<fn(T)>);

impl<'o, T> WriteRegionSlot<'o, T> {
    pub fn fill(self, value: T) {
        let t: NonNull<MaybeUninit<T>> = self.0.cast();
        unsafe {
            // Safety: we know the pointer is valid due to the lenght check and
            // the region creation invariants. Plus we know the memory is uninit
            // - so we do not forget any valid value
            t.write(MaybeUninit::new(value))
        };
        *self.0 = unsafe {
            // Safety: lenght check ensure this is in the slice
            t.add(1)
        }
        .cast();
        *self.1 -= 1;
    }
}

#[cfg(test)]
#[cfg(target_has_atomic = "ptr")]
mod tests {
    use crate::{Buffer, sync::Atomics};

    /// Popping only some of a read region advances `head` by the number
    /// actually popped, not by the region's length.
    #[test]
    fn partial_read_advances_by_popped_count() {
        let mut buf = Buffer::<i32, 8, Atomics>::new();
        let (mut w, mut r) = buf.split();
        for i in 0..5 {
            w.push(i).unwrap();
        }

        r.read(|region| {
            assert_eq!(region.len(), 5);
            assert!(!region.is_empty());
            assert_eq!(region.pop(), Some(0));
            assert_eq!(region.pop(), Some(1));
            // on region drop (end of closure): head += 2 only
        });

        assert_eq!(r.available(), 3);
        assert_eq!(r.pop(), Some(2));
    }

    /// Pushing only some of a write region advances `tail` by the number
    /// actually pushed.
    #[test]
    fn partial_write_advances_by_pushed_count() {
        let mut buf = Buffer::<i32, 8, Atomics>::new();
        let (mut w, mut r) = buf.split();

        w.write(|region| {
            assert_eq!(region.len(), 8);
            region.push(10).unwrap();
            region.push(11).unwrap();
            // on region drop (end of closure): tail += 2 only
        });

        assert_eq!(r.available(), 2);
        assert_eq!(r.pop(), Some(10));
        assert_eq!(r.pop(), Some(11));
        assert_eq!(r.pop(), None);
    }
}
