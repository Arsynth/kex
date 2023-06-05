//! `kex` - library for streamed hex dumping

use ascii::*;
use std::io::{Read, Result, Write};

pub const DEFAULT_BYTES_PER_ROW: usize = 16;

/// The topmost struct for data output
pub struct Printer<A: AddressFormatting, B: ByteFormatting, T: ByteFormatting> {
    /// Where to print data
    out: Box<dyn Write>,

    /// Base address to print.
    current_address: usize,
    config: Config<A, B, T>,

    text_write: TextWrite,
}

impl<A: AddressFormatting, B: ByteFormatting, T: ByteFormatting> Printer<A, B, T> {
    /// Customized constructor.
    /// `out` - place to ouput string.
    ///
    /// `start_address` - start address to print.
    ///
    /// `config` - formatting configuration.
    ///
    /// `Printer` does no assumptions on `start_address` where to start reading data,
    /// it just recieving data chunks in `push(...)` function, then increments the `start_address`
    pub fn new(
        out: Box<dyn Write>,
        start_address: usize,
        config: Config<A, B, T>,
    ) -> Printer<A, B, T> {
        let text_write = TextWrite::new(config.bytes_per_row);
        Printer {
            out,
            current_address: start_address,
            config,
            text_write,
        }
    }

    /// Finalize manually. Prints last unfinished line with paddings. By default it called in `drop()`
    pub fn finish(mut self) {
        self.print_last_line();
    }
}

impl<A: AddressFormatting, B: ByteFormatting, T: ByteFormatting> Printer<A, B, T> {
    fn print_last_line(&mut self) {
        if !self.text_write.has_data() {
            return;
        }

        let bpr = self.config.bytes_per_row;
        let rem = self.current_address % bpr;
        let fill_count = bpr - rem;

        let grouping = self.calc_grouping();
        let spaces = {
            let g_rem = rem % grouping;
            let inc = if g_rem == 0 && rem != 0 { 1 } else { 0 };
            (rem - g_rem) / grouping + inc
        };

        let pad = self.config.fmt.byte.padding_string(fill_count) + &" ".repeat(spaces);
        _ = self.out.write_all(pad.as_bytes());
        _ = self.out.write(b" ");

        _ = self
            .out
            .write_all(self.config.third_column_sep.0.as_bytes());
        _ = self
            .text_write
            .flush(&mut self.out, &mut self.config.fmt.text);

        let pad = self.config.fmt.text.padding_string(fill_count);
        _ = self.out.write_all(pad.as_bytes());
        _ = self
            .out
            .write_all(self.config.third_column_sep.1.as_bytes());

        let _ = self.out.write(b"\n");
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

impl<A: AddressFormatting, B: ByteFormatting, T: ByteFormatting> Printer<A, B, T> {
    pub fn push(&mut self, bytes: &[u8]) -> Result<()> {
        use std::cmp::min;

        let grouping = self.calc_grouping();
        let addr_fmt = &mut self.config.fmt.addr;
        let byte_fmt = &mut self.config.fmt.byte;
        let txt_fmt = &mut self.config.fmt.text;

        let mut tmp = bytes;

        let bpr = self.config.bytes_per_row;

        while tmp.len() > 0 {
            let addr = self.current_address;
            // Row remainder
            let r_rem = addr % bpr;
            // Group remainder
            let g_rem = r_rem % grouping;

            let fill_count = min(min(grouping - g_rem, bpr - r_rem), tmp.len());

            // Check if we need to print address
            if r_rem == 0 {
                let addr_str = addr_fmt.format(self.current_address);
                self.out.write_all(addr_str.as_bytes())?;

                self.out.write_all(b" ")?;
            }

            let out_bytes = &tmp[..fill_count];

            let data_str = byte_fmt.format(out_bytes);
            self.out.write_all(data_str.as_bytes())?;

            self.current_address += fill_count;

            let need_newline = fill_count + r_rem >= bpr;
            let need_group_sep = !need_newline & (fill_count + g_rem >= grouping);

            if need_newline {
                self.out.write_all(b" ")?;
                self.out
                    .write_all(self.config.third_column_sep.0.as_bytes())?;
            } else if need_group_sep {
                self.out.write_all(b" ")?;
            }

            let _ = self.text_write.write(out_bytes, &mut self.out, txt_fmt)?;

            if need_newline {
                self.out
                    .write_all(self.config.third_column_sep.1.as_bytes())?;
                self.out.write_all(b"\n")?;
            }

            tmp = &tmp[fill_count..];
        }

        Ok(())
    }
}

impl<A: AddressFormatting + Default, B: ByteFormatting + Default, T: ByteFormatting + Default>
    Printer<A, B, T>
{
    pub fn default_with(out: Box<dyn Write>, start_address: usize) -> Printer<A, B, T> {
        Self::new(out, start_address, Config::<A, B, T>::default())
    }
}

impl Printer<AddressFormatter, ByteFormatter, CharFormatter> {
    pub fn default_fmt_with(
        out: Box<dyn Write>,
        start_address: usize,
    ) -> Printer<AddressFormatter, ByteFormatter, CharFormatter> {
        Self::new(
            out,
            start_address,
            Config::<AddressFormatter, ByteFormatter, CharFormatter>::default(),
        )
    }
}

impl<A: AddressFormatting, B: ByteFormatting, T: ByteFormatting> Drop for Printer<A, B, T> {
    fn drop(&mut self) {
        self.print_last_line();
    }
}

/// Configuration of formatting
pub struct Config<A: AddressFormatting, B: ByteFormatting, T: ByteFormatting> {
    fmt: Formatters<A, B, T>,

    bytes_per_row: usize,

    byte_grouping: usize,
    third_column_sep: (String, String),
}

impl<A: AddressFormatting, B: ByteFormatting, T: ByteFormatting> Config<A, B, T> {
    /// Create new config.
    /// `bytes_per_row` should be greater than zero, otherwise it defaults to [`DEFAULT_BYTES_PER_ROW`]
    pub fn new(
        fmt: Formatters<A, B, T>,
        bytes_per_row: usize,
        byte_grouping: usize,
        third_column_sep: (String, String),
    ) -> Self {
        let bpr = if bytes_per_row == 0 {
            DEFAULT_BYTES_PER_ROW
        } else {
            bytes_per_row
        };

        Self {
            fmt,
            bytes_per_row: bpr,
            byte_grouping,
            third_column_sep,
        }
    }
}

impl<A: AddressFormatting + Default, B: ByteFormatting + Default, T: ByteFormatting + Default>
    Default for Config<A, B, T>
{
    fn default() -> Config<A, B, T> {
        let fmt: Formatters<A, B, T> = Formatters::new(A::default(), B::default(), T::default());
        Self {
            fmt,
            bytes_per_row: 16,
            byte_grouping: 4,
            third_column_sep: ("|".to_string(), "|".to_string()),
        }
    }
}

/// Formatters for address (first column), bytes (second column), and text (third column)
pub struct Formatters<A: AddressFormatting, B: ByteFormatting, T: ByteFormatting> {
    addr: A,
    byte: B,
    text: T,
}

impl<A: AddressFormatting, B: ByteFormatting, T: ByteFormatting> Formatters<A, B, T> {
    pub fn new(addr: A, byte: B, text: T) -> Self {
        Self { addr, byte, text }
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
        out: &mut Box<dyn Write>,
        fmt: &mut T,
    ) -> Result<usize> {
        use std::cmp::min;

        let len = min(bytes.len(), self.buf.len() - self.avail);

        let mut tmp = &bytes[..];
        tmp.read_exact(&mut self.buf[self.avail..self.avail + len])?;

        self.avail += len;

        if self.avail == self.buf.len() {
            let s = fmt.format(&self.buf);
            let _ = out.write_all(s.as_bytes())?;

            self.avail = 0;
        }

        Ok(len)
    }

    fn flush<T: ByteFormatting>(&mut self, out: &mut Box<dyn Write>, fmt: &mut T) -> Result<()> {
        if self.avail > 0 {
            let s = fmt.format(&self.buf[..self.avail]);
            out.write_all(s.as_bytes())?;

            self.avail = 0;
        }

        Ok(())
    }

    fn has_data(&self) -> bool {
        self.avail != 0
    }
}

/// Used for address formatting (`first` column)
pub trait AddressFormatting {
    fn format(&self, addr: usize) -> String;
}

/// Used for bytes formatting (both for `second` and `third` columns)
pub trait ByteFormatting {
    fn format(&mut self, bytes: &[u8]) -> String;
    
    /// For the flexibility purpose (for example, you may need add ANSI color codes to output data), 
    /// there are no strict checking for printable byte format length. 
    /// Getting the spacing string with incorrect length will result with inaccurate output
    fn padding_string(&mut self, byte_count: usize) -> String;
}

/// Builtin address formatter
pub struct AddressFormatter {
    min_width: usize,
}

impl AddressFormatter {
    pub fn new(min_width: usize) -> AddressFormatter {
        Self { min_width }
    }
}

impl Default for AddressFormatter {
    fn default() -> Self {
        Self { min_width: 8 }
    }
}

impl AddressFormatting for AddressFormatter {
    fn format(&self, addr: usize) -> String {
        format!("{:0width$x}", addr, width = self.min_width)
    }
}

/// Builtin byte formatter (used for `second` column by default)
pub struct ByteFormatter {}

impl ByteFormatter {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for ByteFormatter {
    fn default() -> Self {
        Self {}
    }
}

impl ByteFormatting for ByteFormatter {
    fn format(&mut self, bytes: &[u8]) -> String {
        let strs: Vec<String> = bytes.iter().map(|b| format!("{:02x}", b)).collect();
        strs.join("")
    }

    fn padding_string(&mut self, byte_count: usize) -> String {
        "..".repeat(byte_count)
    }
}

/// Builtin byte formatter (used for `third` column by default)
pub struct CharFormatter {}

impl CharFormatter {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for CharFormatter {
    fn default() -> Self {
        Self {}
    }
}

impl ByteFormatting for CharFormatter {
    fn format(&mut self, bytes: &[u8]) -> String {
        let strs: Vec<String> = bytes
            .iter()
            .map(|b| match AsciiChar::from_ascii(*b) {
                Ok(chr) => chr.to_string(),
                Err(_) => ".".to_string(),
            })
            .collect();
        strs.join("")
    }

    fn padding_string(&mut self, byte_count: usize) -> String {
        " ".repeat(byte_count)
    }
}
