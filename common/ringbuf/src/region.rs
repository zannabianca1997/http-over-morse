//! Contiguous views into buffers

use core::mem::MaybeUninit;

use crate::sync::{
    DefaultSynchronization, MaybeCriticalSection, NoCriticalSection, Synchronization,
};

/// A contiguous view in a buffer
struct Region<
    'o,
    T,
    Sync: Synchronization = DefaultSynchronization,
    CS: MaybeCriticalSection<Sync> = NoCriticalSection,
> {
    /// Length of the region at its creation
    ///
    /// The region only ever shrinks from the front
    original_len: usize,
    /// Slice of the buffer this region has unique use of
    region: &'o mut [MaybeUninit<T>],
    /// Head or tail of the buffer, the one set to the start of the region
    start_index_storage: &'o Sync::Usize,
    /// Possible critical section in which this region is used, to avoid
    /// declaring one during `drop`
    cs: CS,
}

impl<'o, T, Sync: Synchronization, CS: MaybeCriticalSection<Sync>> Region<'o, T, Sync, CS> {
    fn new(region: &'o mut [MaybeUninit<T>], start_index_storage: &'o Sync::Usize, cs: CS) -> Self {
        Self {
            original_len: region.len(),
            region,
            start_index_storage,
            cs,
        }
    }

    fn len(&self) -> usize {
        self.region.len()
    }

    fn is_empty(&self) -> bool {
        self.region.is_empty()
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
impl<'o, T, Sync: Synchronization, CS: MaybeCriticalSection<Sync>> Drop
    for Region<'o, T, Sync, CS>
{
    fn drop(&mut self) {
        // The region only shrinks from the front, so the number of slots read
        // or written is the difference between the original and current length.
        let used = self.original_len - self.region.len();
        // Update the start of the region in the buffer
        self.cs
            .free(|cs| Sync::wrapping_add(cs, self.start_index_storage, used))
    }
}

pub struct ReadRegion<
    'o,
    T,
    Sync: Synchronization = DefaultSynchronization,
    CS: MaybeCriticalSection<Sync> = NoCriticalSection,
>(Region<'o, T, Sync, CS>);

impl<'b, T, Sync: Synchronization, CS: MaybeCriticalSection<Sync>> ReadRegion<'b, T, Sync, CS> {
    pub(crate) fn new(
        region: &'b mut [MaybeUninit<T>],
        start_index_storage: &'b Sync::Usize,
        cs: CS,
    ) -> Self {
        Self(Region::new(region, start_index_storage, cs))
    }

    /// Remove an element from the region
    pub fn pop(&mut self) -> Option<T> {
        // Take out a value, updating the slice
        let value = self.0.region.split_off_first_mut()?;

        Some(unsafe {
            // Safety: ReadRegion can only be created by a reader, that ensures
            // it is pointing to fully initialized memory
            value.assume_init_read()
        })
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

pub struct WriteRegion<
    'o,
    T,
    Sync: Synchronization = DefaultSynchronization,
    CS: MaybeCriticalSection<Sync> = NoCriticalSection,
>(Region<'o, T, Sync, CS>);

impl<'o, T, Sync: Synchronization, CS: MaybeCriticalSection<Sync>> WriteRegion<'o, T, Sync, CS> {
    pub(crate) fn new(
        region: &'o mut [MaybeUninit<T>],
        start_index_storage: &'o Sync::Usize,
        cs: CS,
    ) -> Self {
        Self(Region::new(region, start_index_storage, cs))
    }

    /// Add an element into the region
    pub fn push(&mut self, item: T) -> Result<(), T> {
        // Take a slot, updating the slice
        let Some(value) = self.0.region.split_off_first_mut() else {
            return Err(item);
        };

        value.write(item);

        Ok(())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
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
