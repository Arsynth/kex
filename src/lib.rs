use ascii::*;
use std::io::{Read, Result, Write};

pub const DEFAULT_BYTES_PER_ROW: usize = 16;

pub struct Printer<A: AddressFormatting, B: ByteFormatting, T: ByteFormatting> {
    out: Box<dyn Write>,

    address: usize,
    config: Config<A, B, T>,

    text_write: TextWrite,
    is_finished: bool,
}

impl<A: AddressFormatting, B: ByteFormatting, T: ByteFormatting> Printer<A, B, T> {
    pub fn new(
        out: Box<dyn Write>,
        base_address: usize,
        config: Config<A, B, T>,
    ) -> Printer<A, B, T> {
        let text_write = TextWrite::new(config.bytes_per_row);
        Printer {
            out,
            address: base_address,
            config,
            text_write,
            is_finished: false,
        }
    }

    fn finish(&mut self) {
        if self.is_finished {
            return;
        }

        let bpr = self.config.bytes_per_row;
        let rem = self.address % bpr;
        let fill_count = bpr - rem;

        let pad = self.config.fmt.byte.padding_string(fill_count);
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
}

impl<A: AddressFormatting, B: ByteFormatting, T: ByteFormatting> Printer<A, B, T> {
    pub fn push(&mut self, bytes: &[u8]) -> Result<()> {
        use std::cmp::min;

        let addr_fmt = &mut self.config.fmt.addr;
        let byte_fmt = &mut self.config.fmt.byte;
        let txt_fmt = &mut self.config.fmt.text;

        let mut tmp = bytes;

        let bpr = self.config.bytes_per_row;

        while tmp.len() > 0 {
            let addr = self.address;

            let rem = addr % bpr;

            let fill_count = min(bpr - rem, tmp.len());

            if rem == 0 {
                let addr_str = addr_fmt.format(self.address);
                self.out.write_all(addr_str.as_bytes())?;

                self.out.write_all(b" ")?;
            }

            let out_bytes = &tmp[..fill_count];

            let data_str = byte_fmt.format(out_bytes);
            self.out.write_all(data_str.as_bytes())?;

            self.address += fill_count;

            let need_newline = fill_count + rem >= bpr;

            if need_newline {
                self.out.write_all(b" ")?;
                self.out
                    .write_all(self.config.third_column_sep.0.as_bytes())?;
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

impl<A: AddressFormatting, B: ByteFormatting, T: ByteFormatting> Drop for Printer<A, B, T> {
    fn drop(&mut self) {
        self.finish()
    }
}

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
}

pub trait AddressFormatting {
    fn format(&self, addr: usize) -> String;
}

pub trait ByteFormatting {
    fn format(&mut self, bytes: &[u8]) -> String;
    fn padding_string(&mut self, byte_count: usize) -> String;
}

pub struct AddressFormatter {
    min_width: usize,
}

impl AddressFormatter {
    pub fn new(min_width: usize) -> AddressFormatter {
        Self { min_width }
    }
}

impl AddressFormatting for AddressFormatter {
    fn format(&self, addr: usize) -> String {
        format!("{:0width$x}", addr, width = self.min_width)
    }
}

pub struct ByteFormatter {}

impl ByteFormatter {
    pub fn new() -> Self {
        Self {}
    }
}

impl ByteFormatting for ByteFormatter {
    fn format(&mut self, bytes: &[u8]) -> String {
        let strs: Vec<String> = bytes.iter().map(|b| format!("{:02x}", b)).collect();
        strs.join("")
    }

    fn padding_string(&mut self, byte_count: usize) -> String {
        "  ".repeat(byte_count)
    }
}

pub struct CharFormatter {}

impl CharFormatter {
    pub fn new() -> Self {
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
