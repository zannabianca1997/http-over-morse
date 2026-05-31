use arduino_hal::prelude::_embedded_hal_serial_Write as Write;

/// A running process
pub trait Process {
    /// The type of item produced
    type Ok;

    /// Error type
    type Err;

    /// Do some work without blocking
    fn poll(&mut self) -> nb::Result<Self::Ok, Self::Err>;
}

/// Uninhabited type for processes that does not produce anything
pub enum NoItemProduced {}

impl<T> Process for T
where
    T: arduino_hal::prelude::_embedded_hal_serial_Read<u8>,
{
    type Ok = u8;
    type Err = T::Error;

    fn poll(&mut self) -> nb::Result<Self::Ok, Self::Err> {
        self.read()
    }
}

/// Write all bytes from a process to something implementing [`Write`]
pub struct PipeToWriter<Process, Writer> {
    writer: Writer,
    process: Process,
    writing: Option<u8>,
}

impl<P, W> PipeToWriter<P, W> {
    pub fn new(writer: W, process: P) -> Self {
        Self {
            writer,
            process,
            writing: None,
        }
    }
}

impl<P, W> Process for PipeToWriter<P, W>
where
    P: Process<Ok = u8>,
    W: Write<u8>,
{
    type Ok = NoItemProduced;
    type Err = PipeToWriterError<P::Err, W::Error>;

    fn poll(&mut self) -> nb::Result<Self::Ok, Self::Err> {
        loop {
            // Ensure the writing char is empty
            while let Some(writing) = self.writing {
                match self.writer.write(writing) {
                    Ok(()) => self.writing = None,
                    Err(err) => return Err(err.map(PipeToWriterError::Writer)),
                }
            }
            // Get a new char to write
            match self.process.poll() {
                Ok(ch) => self.writing = Some(ch),
                Err(err) => return Err(err.map(PipeToWriterError::Process)),
            }
        }
    }
}

pub enum PipeToWriterError<I, S> {
    Process(I),
    Writer(S),
}
