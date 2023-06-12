//! `kex` - library for streamed hex dumping

use std::{cell::RefCell, io::*, rc::Rc};

pub mod config;
pub use config::*;

pub mod format;
pub use format::*;

mod writers;
use writers::*;

const ROW_SEPARATOR: &[u8] = b"\n";

const OUTPUT_LOST_MESSAGE: &str = "Somewhere we lost the output";
const WRITER_LOST_MESSAGE: &str = "Somewhere we lost the writer";

/// The topmost struct for data output
pub struct Printer<
    O: Write,
    A: AddressFormatting + Clone,
    B: ByteFormatting + Clone,
    C: CharFormatting + Clone,
> {
    /// Where to print data
    out: Option<O>,

    /// Base address to print.
    printable_address: usize,

    address_fmt: Option<A>,
    byte_fmt: B,
    bytes_writer: Option<GrouppedWriter>,
    text_writer: Option<TextWriter<C>>,

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
        let text_write = match config.text {
            Some(fmt) => Some(TextWriter::new(fmt, config.byte.bytes_per_row())),
            None => None,
        };
        let groupping = config.byte.groupping();
        let byte_order = config.byte.byte_order();
        Printer {
            out: Some(out),
            printable_address: start_address,
            address_fmt: config.addr,
            byte_fmt: config.byte,
            bytes_writer: Some(GrouppedWriter::new(groupping, byte_order)),
            text_writer: text_write,
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
    pub fn current_address(&self) -> usize {
        self.printable_address
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
        let mut b_writer = self.bytes_writer.take().expect(WRITER_LOST_MESSAGE);

        let in_ref = Rc::new(RefCell::new(self));

        let mut callbacks = Callbacks::new(
            || {
                Self::on_row_started(in_ref.clone())?;
                Ok(())
            },
            |write_res| {
                Self::on_data_written(in_ref.clone(), &write_res)?;
                Ok(())
            },
            || {
                Self::on_row_finished(in_ref.clone())?;
                Ok(())
            },
        );

        let mut tmp = &bytes[..];
        let result = {
            while tmp.len() > 0 {
                let written = b_writer.write(tmp, &mut callbacks)?;
                tmp = &tmp[written..];
            }

            Ok(bytes.len())
        };

        in_ref.borrow_mut().bytes_writer = Some(b_writer);

        result
    }

    fn print_last_line(&mut self) -> Result<()> {
        if self.is_finished {
            return Ok(());
        }

        let mut out = self.out.take().expect(OUTPUT_LOST_MESSAGE);
        let mut b_writer = self.bytes_writer.take().expect(WRITER_LOST_MESSAGE);

        let result = {
            b_writer.flush(|buf, byte_number_in_row| {
                let last = self.byte_fmt.format(buf, byte_number_in_row - buf.len());
                out.write_all(last.as_bytes())?;

                self.printable_address += buf.len();
                let padding = self.byte_fmt.padding_string(byte_number_in_row);
                out.write_all(padding.as_bytes())?;

                out.write_all(&self.byte_fmt.separators().leaidng)?;

                if let Some(text_writer) = &mut self.text_writer {
                    text_writer.write(buf)?;
                }

                Ok(())
            })?;

            if let Some(text_writer) = &mut self.text_writer {
                let text = text_writer.take_result();
                out.write_all(text.as_bytes())?;
            }

            Ok(())
        };

        self.out = Some(out);
        self.bytes_writer = Some(b_writer);

        result
    }

    fn on_row_started(this: Rc<RefCell<&mut Self>>) -> Result<()> {
        let mut this = this.borrow_mut();
        let mut out = this.out.take().expect(OUTPUT_LOST_MESSAGE);

        let result = {
            if let Some(address_fmt) = &this.address_fmt {
                let addr_str = address_fmt.format(this.printable_address);

                out.write_all(&address_fmt.separators().trailing)?;

                out.write_all(addr_str.as_bytes())?;

                out.write_all(&address_fmt.separators().leaidng)?;
            }

            Ok(())
        };

        this.out = Some(out);

        result
    }

    fn on_data_written(this: Rc<RefCell<&mut Self>>, data: &WriteResult) -> Result<()> {
        let mut this = this.borrow_mut();
        let mut out = this.out.take().expect(OUTPUT_LOST_MESSAGE);

        let result = {
            match data {
                WriteResult::Stored(_) => Ok(()),
                WriteResult::ReadyAt(buf, byte_in_row) => {
                    let str = this.byte_fmt.format(&buf[..], *byte_in_row);
                    out.write_all(str.as_bytes())?;

                    if let Some(text_writer) = &mut this.text_writer {
                        text_writer.write(buf)?;
                    }

                    this.printable_address += buf.len();
                    Ok(())
                }
            }
        };

        this.out = Some(out);

        result
    }

    fn on_row_finished(this: Rc<RefCell<&mut Self>>) -> Result<()> {
        let mut this = this.borrow_mut();

        let mut out = this.out.take().expect(OUTPUT_LOST_MESSAGE);

        let result = {
            out.write_all(&this.byte_fmt.separators().leaidng)?;

            if let Some(text_writer) = &mut this.text_writer {
                let text = text_writer.take_result();
                out.write_all(text.as_bytes())?;
            }

            out.write_all(ROW_SEPARATOR)?;

            Ok(())
        };

        this.out = Some(out);

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
