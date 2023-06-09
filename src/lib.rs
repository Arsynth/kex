//! `kex` - library for streamed hex dumping

use std::cmp::min;
use std::io::*;

pub mod config;
pub use config::*;

pub mod format;
pub use format::*;

pub mod adapters;
pub use adapters::*;

const ROW_SEPARATOR: &[u8] = b"\n";
const SPACE: &[u8] = b" ";

/// The topmost struct for data output
pub struct Printer<O: Write, A: AddressFormatting, B: ByteFormatting, T: ByteFormatting> {
    /// Where to print data
    out: Option<O>,

    /// Base address to print.
    printable_address: usize,
    address: usize,
    config: Config<A, B, T>,

    text_write: TextWrite,
}

impl<O: Write, A: AddressFormatting, B: ByteFormatting, T: ByteFormatting> Printer<O, A, B, T> {
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
    pub fn new(out: O, start_address: usize, config: Config<A, B, T>) -> Printer<O, A, B, T> {
        let text_write = TextWrite::new(config.bytes_per_row);
        Printer {
            out: Some(out),
            printable_address: start_address,
            address: 0,
            config,
            text_write,
        }
    }

    /// Finalize manually. Prints last unfinished line with paddings and turns back given output
    pub fn finish(mut self) -> O {
        _ = self.print_last_line();
        self.out.take().unwrap()
    }
}

impl<O: Write, A: AddressFormatting, B: ByteFormatting, T: ByteFormatting> Printer<O, A, B, T> {
    pub fn current_address(&self) -> usize {
        self.printable_address
    }
}

impl<O: Write, A: AddressFormatting, B: ByteFormatting, T: ByteFormatting> Printer<O, A, B, T> {
    fn print_last_line(&mut self) -> Result<()> {
        let third_column_sep = self.config.decorations.third_column_sep.clone();

        if !self.text_write.has_data() {
            return Ok(());
        }

        let mut addr = self.address;
        let bpr = self.config.bytes_per_row;
        let grouping = self.calc_grouping();
        let mut tmp = bpr - (addr % bpr);

        let mut out = self.out.take().unwrap();

        while tmp > 0 {
            // Row remainder
            let r_rem = addr % bpr;
            // Group remainder
            let g_rem = r_rem % grouping;

            let fill_count = min(min(grouping - g_rem, bpr - r_rem), tmp);

            let data_str = self.config.fmt.byte.padding_string(fill_count, bpr - tmp);
            _ = out.write_all(data_str.as_bytes())?;

            addr += fill_count;

            tmp -= fill_count;
        }

        let rem = self.address % bpr;
        let fill_count = bpr - rem;
        _ = out.write(SPACE)?;

        _ = out.write_all(&third_column_sep.0)?;
        _ = self.text_write.flush(&mut out, &mut self.config.fmt.text)?;

        let pad = self.config.fmt.text.padding_string(fill_count, bpr - fill_count);
        _ = out.write_all(pad.as_bytes())?;
        _ = out.write_all(&third_column_sep.1)?;

        let _ = out.write(b"\n")?;

        self.out = Some(out);

        Ok(())
    }

    fn calc_grouping(&self) -> usize {
        let grouping = if self.config.byte_grouping > 0 {
            self.config.byte_grouping
        } else {
            self.config.bytes_per_row
        };
        grouping
    }
}

impl<O: Write, A: AddressFormatting, B: ByteFormatting, T: ByteFormatting> Printer<O, A, B, T> {
    /// Accepts bytes chunk. Immediately prints `first` and `second` columns to `out`,
    /// `third` will printed after `second` column is completely filled, or after finalization.
    pub fn push(&mut self, bytes: &[u8]) -> Result<usize> {
        let third_column_sep = self.config.decorations.third_column_sep.clone();

        let grouping = self.calc_grouping();

        let addr_fmt = &mut self.config.fmt.addr;
        let byte_fmt = &mut self.config.fmt.byte;
        let txt_fmt = &mut self.config.fmt.text;
        let mut tmp = bytes;
        let bpr = self.config.bytes_per_row;

        let mut out = self.out.take().unwrap();

        let result = {
            while tmp.len() > 0 {
                let addr = self.address;
                // Row remainder
                let r_rem = addr % bpr;
                // Group remainder
                let g_rem = r_rem % grouping;

                let fill_count = min(min(grouping - g_rem, bpr - r_rem), tmp.len());

                // Check if we need to print address
                if r_rem == 0 {
                    let addr_str = addr_fmt.format(self.printable_address);
                    let bytes = addr_str.as_bytes();
                    out.write_all(bytes)?;

                    out.write_all(SPACE)?;
                }

                let out_bytes = &tmp[..fill_count];

                let data_str = byte_fmt.format(out_bytes, bpr - r_rem);
                let bytes = data_str.as_bytes();
                out.write_all(bytes)?;

                self.address += fill_count;
                self.printable_address += fill_count;

                let need_newline = fill_count + r_rem >= bpr;
                let need_group_sep = !need_newline & (fill_count + g_rem >= grouping);

                if need_newline {
                    out.write_all(SPACE)?;

                    let bytes = &third_column_sep.0;
                    out.write_all(bytes)?;
                }

                self.text_write.write(out_bytes, &mut out, txt_fmt)?;

                if need_newline {
                    let bytes = &third_column_sep.1;
                    out.write_all(bytes)?;

                    out.write_all(ROW_SEPARATOR)?;
                }

                tmp = &tmp[fill_count..];
            }

            Ok(bytes.len())
        };

        self.out = Some(out);

        result
    }
}

impl<
        O: Write,
        A: AddressFormatting + Default,
        B: ByteFormatting + Default,
        T: ByteFormatting + Default,
    > Printer<O, A, B, T>
{
    pub fn default_with(out: O, start_address: usize) -> Printer<O, A, B, T> {
        Self::new(out, start_address, Config::<A, B, T>::default())
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

impl<O: Write, A: AddressFormatting, B: ByteFormatting, T: ByteFormatting> Write
    for Printer<O, A, B, T>
{
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.push(buf)
    }

    fn flush(&mut self) -> Result<()> {
        self.print_last_line()
    }
}

impl<O: Write, A: AddressFormatting, B: ByteFormatting, T: ByteFormatting> Drop
    for Printer<O, A, B, T>
{
    fn drop(&mut self) {
        _ = self.print_last_line();
    }
}

struct TextWrite {
    buf: Vec<u8>,
    avail: usize,
}

impl TextWrite {
    fn new(max_bytes: usize) -> Self {
        Self {
            buf: vec![0; max_bytes],
            avail: 0,
        }
    }
}

impl TextWrite {
    fn write<T: ByteFormatting>(
        &mut self,
        bytes: &[u8],
        out: &mut dyn Write,
        fmt: &mut T,
    ) -> Result<usize> {
        let len = min(bytes.len(), self.buf.len() - self.avail);

        let mut tmp = &bytes[..];
        tmp.read_exact(&mut self.buf[self.avail..self.avail + len])?;

        self.avail += len;

        if self.avail == self.buf.len() {
            let s = fmt.format(&self.buf, 0);
            let _ = out.write_all(s.as_bytes())?;

            self.avail = 0;
        }

        Ok(len)
    }

    fn flush<T: ByteFormatting>(&mut self, out: &mut dyn Write, fmt: &mut T) -> Result<()> {
        todo!()
    }

    fn has_data(&self) -> bool {
        self.avail != 0
    }
}
