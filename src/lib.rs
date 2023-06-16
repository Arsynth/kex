//! `kex` - library for streamed hex dumping

use std::io::*;

pub mod config;
pub use config::*;

pub mod format;
pub use format::*;

mod streamer;
use streamer::*;

const OUTPUT_LOST_MESSAGE: &str = "Somewhere we lost the output";

/// The topmost struct for data output
pub struct Printer<
    O: Write,
    A: AddressFormatting + Clone,
    B: ByteFormatting + Clone,
    C: CharFormatting + Clone,
> {
    /// Where to print data
    out: Option<O>,

    streamer: Streamer<A, B, C>,

    is_finished: bool,
}

impl<
        O: Write,
        A: AddressFormatting + Clone,
        B: ByteFormatting + Clone,
        C: CharFormatting + Clone,
    > Printer<O, A, B, C>
{
    /// Customized constructor.
    ///
    /// All constructors of the [`Printer`] moves given output. To give it back use `finish(mut self)` function
    ///
    /// `out` - place to ouput string.
    ///
    /// `start_address` - start address to print.
    ///
    /// `config` - formatting configuration.
    ///
    /// `Printer` does no assumptions on `start_address` where to start reading data,
    /// it just recieving data chunks in `push(...)` function, then increments the `start_address`
    pub fn new(out: O, start_address: usize, config: Config<A, B, C>) -> Printer<O, A, B, C> {
        Printer {
            out: Some(out),
            streamer: Streamer::new(config.addr, config.byte, config.text, start_address),
            is_finished: false,
        }
    }

    /// Finalize manually. Prints last unfinished line with paddings and turns back given output
    pub fn finish(mut self) -> O {
        _ = self.print_last_line();
        self.is_finished = true;
        self.out.take().unwrap()
    }
}

impl<
        O: Write,
        A: AddressFormatting + Clone,
        B: ByteFormatting + Clone,
        C: CharFormatting + Clone,
    > Printer<O, A, B, C>
{
    /// Accepts bytes chunk. Immediately prints `first` and `second` columns to `out`,
    /// `third` will printed after `second` column is completely filled, or after finalization.
    pub fn push(&mut self, bytes: &[u8]) -> Result<usize> {
        let mut out = self.out.take().expect(OUTPUT_LOST_MESSAGE);

        self.streamer.push(bytes, &mut out)?;

        self.out = Some(out);

        Ok(bytes.len())
    }

    fn print_last_line(&mut self) -> Result<()> {
        if self.is_finished {
            return Ok(());
        }

        let mut out = self.out.take().expect(OUTPUT_LOST_MESSAGE);

        let result = self.streamer.write_tail(&mut out);

        self.out = Some(out);

        result
    }
}

impl<
        O: Write,
        A: AddressFormatting + Clone + Default,
        B: ByteFormatting + Clone + Default,
        C: CharFormatting + Clone + Default,
    > Printer<O, A, B, C>
{
    pub fn default_with(out: O, start_address: usize) -> Printer<O, A, B, C> {
        Self::new(out, start_address, Config::<A, B, C>::default())
    }
}

impl<O: Write> Printer<O, AddressFormatter, ByteFormatter, CharFormatter> {
    pub fn default_fmt_with(
        out: O,
        start_address: usize,
    ) -> Printer<O, AddressFormatter, ByteFormatter, CharFormatter> {
        Self::new(
            out,
            start_address,
            Config::<AddressFormatter, ByteFormatter, CharFormatter>::default(),
        )
    }
}

impl<
        O: Write,
        A: AddressFormatting + Clone,
        B: ByteFormatting + Clone,
        C: CharFormatting + Clone,
    > Write for Printer<O, A, B, C>
{
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.push(buf)
    }

    fn flush(&mut self) -> Result<()> {
        self.print_last_line()
    }
}

impl<
        O: Write,
        A: AddressFormatting + Clone,
        B: ByteFormatting + Clone,
        C: CharFormatting + Clone,
    > Drop for Printer<O, A, B, C>
{
    fn drop(&mut self) {
        _ = self.print_last_line();
    }
}
