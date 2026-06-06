use core::num::NonZero;

use embedded_hal::digital::PinState;

use crate::Time;

/// Represent a segment on the wire - either high or low
///
/// A segment is a combination of the state of the wire - high or low - and of a
/// length. Maximum representable length is 127 - operations are saturating.
#[derive(Debug, Clone, Copy)]
pub struct Segment(NonZero<u8>);

const LEN_MASK: u8 = 0b0111_1111;
pub const MAX_LENGTH: NonZero<Time> = NonZero::new(LEN_MASK as _).unwrap();

impl Segment {
    /// Pack state and length inside a segment
    pub const fn pack(state: PinState, len: NonZero<Time>) -> Self {
        // a bit lengthy, but as long as const ord is not here we have to do to
        // keep the constness
        let len = if len.get() > MAX_LENGTH.get() {
            MAX_LENGTH.get()
        } else {
            len.get()
        } as u8;

        let pin_bit = match state {
            PinState::Low => 0b0000_0000,
            PinState::High => 0b1000_0000,
        };
        Self(unsafe {
            // Safety: len is nonzero, so at least one of its bit is set. If
            // that's over the first seven bits, they have all been set by the
            // if before. So at least one of the last seven bit is always 1
            NonZero::new_unchecked(pin_bit | len)
        })
    }

    /// Unpack a segment into state and length
    pub const fn unpack(self) -> (PinState, NonZero<Time>) {
        let len = unsafe {
            // Safety: the only safe way of obtaining a segment is through the
            // constructor, and per the safety comment on that one of the last
            // seven bit must have been 1
            NonZero::new_unchecked((self.0.get() & LEN_MASK) as Time)
        };
        let pin_state = if self.0.get() & !LEN_MASK != 0 {
            PinState::High
        } else {
            PinState::Low
        };
        (pin_state, len)
    }
}
