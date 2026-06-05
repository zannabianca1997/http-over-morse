use core::mem;

use morse::Prosign;

pub enum FrameEvent {
    /// Header received
    ///
    /// After this the header can be retrieved with the `header` function
    HeaderReceived,
    /// Start of payload
    PayloadStart,
    /// Byte in the payload
    Payload(u8),
    /// End of payload
    PayloadEnd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FramingProsigns {
    FrameStart = Prosign::Attention as u8,
    PayloadStart = Prosign::Break as u8,
    PayloadEnd = Prosign::Out as u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ConfirmationProsigns {
    Confirm = Prosign::Acknowledge as u8,
    Negative = Prosign::NegativeAcknowledge as u8,
}

impl From<FramingProsigns> for Prosign {
    fn from(value: FramingProsigns) -> Self {
        unsafe {
            // Safety: this is a sub-enum
            mem::transmute(value)
        }
    }
}

impl TryFrom<Prosign> for FramingProsigns {
    type Error = ConfirmationProsigns;
    fn try_from(value: Prosign) -> Result<Self, Self::Error> {
        match value {
            Prosign::Attention | Prosign::Break | Prosign::Out => Ok(unsafe {
                // Safety: these variants have the same representation
                mem::transmute(value)
            }),
            Prosign::Acknowledge | Prosign::NegativeAcknowledge => Err(unsafe {
                // Safety: these variants have the same representation in the
                // confirmation subset
                mem::transmute(value)
            }),
        }
    }
}
