//! Contiguous views into buffers

use core::mem::MaybeUninit;

use crate::sync::{
    DefaultSynchronization, MaybeCriticalSection, NoCriticalSection, Synchronization,
};

/// A contiguous view in a buffer
struct Region<
    'b,
    T,
    Sync: Synchronization = DefaultSynchronization,
    CS: MaybeCriticalSection<Sync> = NoCriticalSection,
> {
    /// Start of the region at it's creation
    original_start: *const MaybeUninit<T>,
    /// Slice of the buffer this region has unique use of
    region: &'b mut [MaybeUninit<T>],
    /// Head or tail of the buffer, the one set to the start of the region
    start_index_storage: &'b Sync::Usize,
    /// Possible critical section in which this region is used, to avoid
    /// declaring one during `drop`
    cs: CS,
}

impl<'o, T, Sync: Synchronization, CS: MaybeCriticalSection<Sync>> Region<'o, T, Sync, CS> {
    fn new(region: &'o mut [MaybeUninit<T>], start_index_storage: &'o Sync::Usize, cs: CS) -> Self {
        Self {
            original_start: region.as_ptr_range().start,
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
// Avoiding this looks impossible without runtime overhead...
impl<'b, 'o, T, Sync: Synchronization, CS: MaybeCriticalSection<Sync>> Drop
    for Region<'o, T, Sync, CS>
{
    fn drop(&mut self) {
        // Obtain the new start of the region
        let new_start = self.region.as_ptr_range().start;
        // Calculate how many elements have been either read or written
        let used = unsafe {
            // Safety: `original_start` can be set only by `new` and it's part
            // of the slice
            new_start.offset_from_unsigned(self.original_start)
        };
        // Update the start of the region in the buffer
        self.cs
            .free(|cs| Sync::increment(cs, self.start_index_storage, used))
    }
}

pub struct ReadRegion<
    'o,
    T,
    Sync: Synchronization = DefaultSynchronization,
    CS: MaybeCriticalSection<Sync> = NoCriticalSection,
>(Region<'o, T, Sync, CS>);

impl<'o, T, Sync: Synchronization, CS: MaybeCriticalSection<Sync>> ReadRegion<'o, T, Sync, CS> {
    pub(crate) fn new(
        region: &'o mut [MaybeUninit<T>],
        start_index_storage: &'o Sync::Usize,
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
