use process::Process;

use crate::events::FrameEvent;

/// Frame aware processes that can be instructed to skip the current frame
pub trait SkipCurrentFrame: Process {
    /// Skip the current frame
    ///
    /// The process will skip the current frame, and do not produce any more
    /// items until the next frame starts.
    fn skip_current_frame(&mut self);
}

/// Process that can provide the header of the current frame
pub trait WithHeader: Process<Ok = FrameEvent> {
    type Header;

    /// Get the header of the current frame
    ///
    /// Will return [`Some`] only between [`FrameEvent::HeaderReceived`] and
    /// [`FrameEvent::PayloadStart`]
    fn header(&self) -> Option<&Self::Header>;
}
