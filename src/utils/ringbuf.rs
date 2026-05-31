//! Ringbuffer implementation
//!
//! Many of the implementations online use atomics :(
//!
//! Implementation is loosely based on
//! [fring](https://docs.rs/fring/latest/fring/)

use core::{cell::Cell, mem::MaybeUninit};

use avr_device::interrupt::{CriticalSection, Mutex, free};

/// A ring buffer for type T and with capacity CAP
///
/// The only way of interacting with this is through [`Writer`] and [`Reader`]
pub struct Buffer<T, const CAP: usize> {
    /// Content of the buffer
    ///
    /// All the elements from `head % CAP` included onward, possibly wrapping
    /// around up to `tail % CAP` excluded are initialized.
    content: [Cell<MaybeUninit<T>>; CAP],
    /// Head of the buffer
    ///
    /// Points to the oldest filled slot.
    ///
    /// Can surpass CAP, as it is wrapped around on access.
    head: Mutex<Cell<usize>>,
    /// Tail of the buffer
    ///
    /// Points to the oldest unfilled slot.
    ///
    /// Can surpass CAP, as it is wrapped around on access.
    tail: Mutex<Cell<usize>>,
}

/// Writable view into a buffer
///
/// For this to be safe to interact with, only a single [`Writer`] must exist at
/// the same time
pub struct Writer<'b, T, const CAP: usize, CS> {
    buffer: &'b Buffer<T, CAP>,
    cs: CS,
}

/// Readable view into a buffer
///
/// For this to be safe to interact with, only a single [`Reader`] must exist at
/// the same time
pub struct Reader<'b, T, const CAP: usize, CS> {
    buffer: &'b Buffer<T, CAP>,
    cs: CS,
}

/// Marker type for handlers that need to declare their own critical sections
pub struct NoCriticalSection;

impl<T, const CAP: usize> Buffer<T, CAP> {
    pub const fn new() -> Self {
        // Force a compile-time failure if the buffer length is not a power of
        // 2.
        //
        // A length equal to a power of two enable us to easy wrap around tail
        // and head via precalculated bitmasks.
        const {
            assert!(
                (CAP != 0) && ((CAP - 1) & CAP == 0),
                "buffer size must be a power of 2"
            )
        };

        Self {
            content: unsafe {
                // Safety: The MaybeUninit can be pushed inside the array, as
                // arrays of MaybeUninit have no requirements. See also
                // MaybeUninit::transpose.
                //
                // Then it can be pushed inside the Cell, as Cell has the same
                // requirements of the inner type.
                //
                // This is more or less us pointing at that memory and telling
                // the compiler what it can and can't assume.
                MaybeUninit::uninit().assume_init()
            },
            head: Mutex::new(Cell::new(0)),
            tail: Mutex::new(Cell::new(0)),
        }
    }

    /// Get a writing handle to this buffer
    ///
    /// Safety: the caller must ensure that only a single [`Writer`] pointing to
    /// the same buffer exists.
    pub const unsafe fn writer(&self) -> Writer<'_, T, CAP, NoCriticalSection> {
        Writer {
            buffer: self,
            cs: NoCriticalSection,
        }
    }

    /// Get a writing handle to this buffer while a critical section
    ///
    /// This will enable the handle to avoid declaring unnecessary critical
    /// sections.
    ///
    /// Safety: the caller must ensure that only a single [`Writer`] pointing to
    /// the same buffer exists, even while in a critical section (the other
    /// might be halfway writing)
    pub const unsafe fn writer_cs<'b, 's>(
        &'b self,
        cs: CriticalSection<'s>,
    ) -> Writer<'b, T, CAP, CriticalSection<'s>> {
        Writer { buffer: self, cs }
    }

    /// Get a reading handle to this buffer
    ///
    /// Safety: the caller must ensure that only a single [`Reader`] pointing to
    /// the same buffer exists.
    pub const unsafe fn reader(&self) -> Reader<'_, T, CAP, NoCriticalSection> {
        Reader {
            buffer: self,
            cs: NoCriticalSection,
        }
    }

    /// Get a reading handle to this buffer while in a critical section
    ///
    /// This will enable the handle to avoid declaring unnecessary critical
    /// sections.
    ///
    /// Safety: the caller must ensure that only a single [`Reader`] pointing to
    /// the same buffer exists.
    pub const unsafe fn reader_cs<'b, 's>(
        &'b self,
        cs: CriticalSection<'s>,
    ) -> Reader<'b, T, CAP, CriticalSection<'s>> {
        Reader { buffer: self, cs }
    }

    fn get_head_and_tail<CS: MaybeCriticalSection>(&self, cs: &CS) -> (usize, usize) {
        cs.free(|cs| {
            let head = self.head.borrow(cs).get();
            let tail = self.tail.borrow(cs).get();

            (head, tail)
        })
    }
}

impl<T, const CAP: usize, CS> Writer<'_, T, CAP, CS>
where
    CS: MaybeCriticalSection,
{
    /// Start and end of the uninitialized part
    fn fetch_start_and_end(&self) -> (usize, usize) {
        let (head, tail) = self.buffer.get_head_and_tail(&self.cs);

        let start = tail;
        let end = head.wrapping_add(CAP);

        (start, end)
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

    /// Push an element inside the buffer
    ///
    /// If the buffer is full the element is returned
    pub fn push(&mut self, item: T) -> Option<T> {
        let (start, end) = self.fetch_start_and_end();

        if start == end {
            // Buffer is full
            return Some(item);
        }

        // Writing
        self.buffer.content[start & (CAP - 1)].replace(MaybeUninit::new(item));

        // Updating the tail, so that the reader can consume this item
        self.cs
            .free(|cs| self.buffer.tail.borrow(cs).set(start.wrapping_add(1)));

        None
    }
}

impl<T, const CAP: usize, CS> Reader<'_, T, CAP, CS>
where
    CS: MaybeCriticalSection,
{
    /// Start and end of the initialized part
    fn fetch_start_and_end(&self) -> (usize, usize) {
        let (head, tail) = self.buffer.get_head_and_tail(&self.cs);

        let start = head;
        let end = tail;

        (start, end)
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

    /// Read an element from the buffer
    pub fn pop(&mut self) -> Option<T> {
        let (start, end) = self.fetch_start_and_end();

        if start == end {
            // Buffer is empty
            return None;
        }

        // Reading
        let item = self.buffer.content[start & (CAP - 1)].replace(MaybeUninit::uninit());

        // Updating the head, so that the writer can add items in its place
        self.cs
            .free(|cs| self.buffer.head.borrow(cs).set(start.wrapping_add(1)));

        Some(unsafe {
            // Safety: see the guarantees on Buffer::content. The element is
            // part of the initialized section of the buffer.
            item.assume_init()
        })
    }
}

/// As we handle concurrency inside, [`Buffer`] can implement [`Sync`].
///
/// Safety: The tail and heads are checked with critical sections, making them
/// already [`Sync`]. The content is not sync, but both the writer and the
/// reader check they have exclusive access to it before interacting. Finally, T
/// must be [`Send`] as we can read it out from another thread.
unsafe impl<T, const CAP: usize> Sync for Buffer<T, CAP> where T: Send {}

pub trait MaybeCriticalSection {
    fn free<R>(&self, fun: impl FnOnce(CriticalSection) -> R) -> R;
}
impl MaybeCriticalSection for NoCriticalSection {
    fn free<R>(&self, fun: impl FnOnce(CriticalSection) -> R) -> R {
        free(fun)
    }
}
impl MaybeCriticalSection for CriticalSection<'_> {
    fn free<R>(&self, fun: impl FnOnce(CriticalSection) -> R) -> R {
        fun(*self)
    }
}
