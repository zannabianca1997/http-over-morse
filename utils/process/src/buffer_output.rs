//! Buffer output of a process

use ringbuf::Buffer;

use super::Process;

pub struct BufferOutput<Inner, const CAP: usize>
where
    Inner: Process,
{
    inner: Inner,
    buffer: Buffer<Result<Inner::Ok, Inner::Err>, CAP>,
}

impl<Inner, const CAP: usize> BufferOutput<Inner, CAP>
where
    Inner: Process,
{
    pub fn new(inner: Inner) -> Self {
        Self {
            inner,
            buffer: Buffer::new(),
        }
    }
}

impl<Inner, const CAP: usize> Process for BufferOutput<Inner, CAP>
where
    Inner: Process,
{
    type Ok = Inner::Ok;
    type Err = Inner::Err;

    fn poll(&mut self) -> nb::Result<Self::Ok, Self::Err> {
        let (mut writer, mut reader) = self.buffer.split();

        // Read from the source until either the buffer is full or the source
        // would block
        'read: while let Some(mut slot) = writer.slot() {
            let res = match self.inner.poll() {
                Ok(ok) => Ok(ok),
                Err(nb::Error::Other(err)) => Err(err),
                Err(nb::Error::WouldBlock) => break 'read,
            };

            slot.push(res);
        }

        // Give answers from the buffer
        match reader.pop() {
            Some(Ok(ok)) => Ok(ok),
            Some(Err(err)) => Err(nb::Error::Other(err)),
            None => Err(nb::Error::WouldBlock),
        }
    }
}
