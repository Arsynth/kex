//! `kex` - library for streamed hex dumping

use std::io::*;

pub mod config;
pub use config::*;

pub mod format;
pub use format::*;

mod writers;
use writers::*;

const ROW_SEPARATOR: &[u8] = b"\n";
const SPACE: &[u8] = b" ";

const OUTPUT_LOST_MESSAGE: &str = "Somewhere we lost the output";
const CANNOT_WRITE_OUTPUT_MESSAGE: &str = "Could not write to output";

/// The topmost struct for data output
pub struct Printer<O: Write, A: AddressFormatting, B: ByteFormatting, C: CharFormatting> {
    /// Where to print data
    out: Option<O>,

    /// Base address to print.
    printable_address: usize,

    address_fmt: A,
    byte_fmt: B,
    bytes_writer: GrouppedWriter,
    text_writer: TextWriter<C>,

    decorations: Decorations,
}

impl<O: Write, A: AddressFormatting, B: ByteFormatting, C: CharFormatting> Printer<O, A, B, C> {
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
        let text_fmt = config.fmt.text;

        let text_write = TextWriter::new(text_fmt, config.fmt.byte.bytes_per_row());
        Printer {
            out: Some(out),
            printable_address: start_address,
            address_fmt: config.fmt.addr,
            byte_fmt: config.fmt.byte,
            bytes_writer: GrouppedWriter::new(
                config.fmt.byte.groupping().clone(),
                config.fmt.byte.byte_order(),
            ),
            text_writer: text_write,
            decorations: config.decorations,
        }
    }

    /// Finalize manually. Prints last unfinished line with paddings and turns back given output
    pub fn finish(mut self) -> O {
        _ = self.print_last_line();
        self.out.take().unwrap()
    }
}

impl<O: Write, A: AddressFormatting, B: ByteFormatting, C: CharFormatting> Printer<O, A, B, C> {
    pub fn current_address(&self) -> usize {
        self.printable_address
    }
}

impl<O: Write, A: AddressFormatting, B: ByteFormatting, C: CharFormatting> Printer<O, A, B, C> {
    /// Accepts bytes chunk. Immediately prints `first` and `second` columns to `out`,
    /// `third` will printed after `second` column is completely filled, or after finalization.
    pub fn push(&mut self, bytes: &[u8]) -> Result<usize> {
        let mut out = self.out.take().expect(OUTPUT_LOST_MESSAGE);

        let mut b_writer = self.bytes_writer;
        let mut tmp = bytes;

        let callbacks = Callbacks::new(
            || {
                let addr_str = self.address_fmt.format(b_writer.address()).as_bytes();
                _ = out.write_all(addr_str).expect(CANNOT_WRITE_OUTPUT_MESSAGE);
            },
            |write_res| {
                match write_res {
                    WriteResult::Stored(_) => (),
                    WriteResult::ReadyAt(buf, byte_in_row) => {
                        let str = self.byte_fmt.format(buf, byte_in_row);
                        out.write_all(str.as_bytes())
                            .expect(CANNOT_WRITE_OUTPUT_MESSAGE);
                    }
                }

                tmp = &tmp[write_res.bytes_processed()..];
            },
            || {
                let decor = self.decorations;
                out.write_all(SPACE).expect(CANNOT_WRITE_OUTPUT_MESSAGE);
                out.write_all(&decor.third_column_sep.0)
                    .expect(CANNOT_WRITE_OUTPUT_MESSAGE);
                self.text_writer
                    .write(bytes, &mut out)
                    .expect(CANNOT_WRITE_OUTPUT_MESSAGE);
                out.write_all(&decor.third_column_sep.1)
                    .expect(CANNOT_WRITE_OUTPUT_MESSAGE);
                out.write_all(ROW_SEPARATOR)
                    .expect(CANNOT_WRITE_OUTPUT_MESSAGE);
            },
        );

        self.out = Some(out);

        Ok(bytes.len())
    }

    fn print_last_line(&mut self) -> Result<()> {
        if !self.text_writer.has_data() {
            return Ok(());
        }

        let mut out = self.out.take().expect(OUTPUT_LOST_MESSAGE);

        self.bytes_writer.flush(|buf| {})?;
        self.text_writer.flush(&mut out)?;

        self.out = Some(out);

        Ok(())
    }
}

impl<
        O: Write,
        A: AddressFormatting + Default,
        B: ByteFormatting + Default,
        C: CharFormatting + Default,
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

impl<O: Write, A: AddressFormatting, B: ByteFormatting, C: CharFormatting> Write
    for Printer<O, A, B, C>
{
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.push(buf)
    }

    fn flush(&mut self) -> Result<()> {
        self.print_last_line()
    }
}

impl<O: Write, A: AddressFormatting, B: ByteFormatting, C: CharFormatting> Drop
    for Printer<O, A, B, C>
{
    fn drop(&mut self) {
        _ = self.print_last_line();
    }
}
