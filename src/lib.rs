//! `kex` - library for streamed hex dumping

use ascii::*;
use std::cmp::min;
use std::io::*;

pub const DEFAULT_BYTES_PER_ROW: usize = 16;
pub const DEFAULT_NUMBER_OF_GROUPS: usize = 4;

const SPACE: &[u8] = b" ";
const NEWLINE: &[u8] = b"\n";

/// The topmost struct for data output
///
/// # Examples
///
/// ```
/// use kex::*;
/// use std::{io::{stdout, Stdout}, fs::File};
/// fn main() {
///     let fmt = Formatters::new(
///         MyAddrFormatter::new(),
///         MyByteFormatter::new(),
///         CharFormatter::new(),
///     );
///     let config = Config::new(fmt, 9, 3, ('\u{1F4A5}'.to_string(), '\u{1F4A8}'.to_string()));
///     let mut printer = Printer::new(Box::new(stdout()), 0, config);
///     let mut _printer = Printer::<Box<Stdout>, AddressFormatter, ByteFormatter, CharFormatter>::default_with(Box::new(stdout()), 0);
///     
///     let mut _printer = Printer::default_fmt_with(Box::new(stdout()), 0);
///
///     let bytes1 = &[222u8, 173, 190, 239];
///     let bytes2 = &[0xfeu8, 0xed, 0xfa];
///     let it_works = &[
///         0x49u8, 0x74, 0x20, 0x77, 0x6f, 0x72, 0x6b, 0x73, 0x21, 0x21, 0x21,
///     ];

///     for _ in 0..10 {
///         _ = printer.push(bytes1);
///     }

///     _ = printer.push(it_works);

///     for _ in 0..11 {
///         _ = printer.push(bytes2);
///     }

///     printer.finish();

///     println!("\nPrinting to vector:\n");

///     let out = Box::new(Vec::<u8>::new());
///     let mut printer = Printer::default_fmt_with(out, 0);

///     _ = printer.push(bytes1);
///     _ = printer.push(it_works);
///     _ = printer.push(bytes2);

///     let out = printer.finish();

///     let result = std::str::from_utf8(&*out).unwrap();
///     println!("{}", result);

///     let file = File::create("target/hexdump.txt").unwrap();
///     let mut printer = Printer::default_fmt_with(file, 0);
///     _ = printer.push(bytes1);
///     _ = printer.push(it_works);
///     _ = printer.push(bytes2);
///     _ = printer.finish();
/// }
///
/// struct MyAddrFormatter {
///     fmt: AddressFormatter,
/// }
///
/// impl MyAddrFormatter {
///     fn new() -> Self {
///         MyAddrFormatter {
///             fmt: AddressFormatter::new(8),
///         }
///     }
/// }

/// impl AddressFormatting for MyAddrFormatter {
///     fn format(&self, addr: usize) -> String {
///         const EMOJI: char = '\u{1F929}';
///         format!("{}{EMOJI}", self.fmt.format(addr))
///     }
/// }

/// struct MyByteFormatter {
///     fmt: ByteFormatter,
/// }
///
/// impl MyByteFormatter {
///     fn new() -> Self {
///         Self {
///             fmt: ByteFormatter::new(),
///         }
///     }
/// }
///
/// impl ByteFormatting for MyByteFormatter {
///     fn format(&mut self, bytes: &[u8]) -> String {
///         self.fmt.format(bytes)
///     }

///     fn padding_string(&mut self, byte_count: usize) -> String {
///         format!("Xx").repeat(byte_count)
///     }
/// }
///
/// ```
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

            let out_bytes = fill_count;

            let data_str = self.config.fmt.byte.padding_string(out_bytes);
            _ = out.write_all(data_str.as_bytes())?;

            addr += fill_count;

            let need_newline = fill_count + r_rem >= bpr;
            let need_group_sep = !need_newline & (fill_count + g_rem >= grouping);

            if need_group_sep {
                _ = out.write_all(SPACE)?;
            }

            tmp -= fill_count;
        }

        let rem = self.address % bpr;
        let fill_count = bpr - rem;
        _ = out.write(SPACE)?;

        _ = out.write_all(third_column_sep.0.as_bytes())?;
        _ = self.text_write.flush(&mut out, &mut self.config.fmt.text)?;

        let pad = self.config.fmt.text.padding_string(fill_count);
        _ = out.write_all(pad.as_bytes())?;
        _ = out.write_all(third_column_sep.1.as_bytes())?;

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

                let data_str = byte_fmt.format(out_bytes);
                let bytes = data_str.as_bytes();
                out.write_all(bytes)?;

                self.address += fill_count;
                self.printable_address += fill_count;

                let need_newline = fill_count + r_rem >= bpr;
                let need_group_sep = !need_newline & (fill_count + g_rem >= grouping);

                if need_newline {
                    out.write_all(SPACE)?;

                    let bytes = third_column_sep.0.as_bytes();
                    out.write_all(bytes)?;
                } else if need_group_sep {
                    out.write_all(SPACE)?;
                }

                self.text_write.write(out_bytes, &mut out, txt_fmt)?;

                if need_newline {
                    let bytes = third_column_sep.1.as_bytes();
                    out.write_all(bytes)?;

                    out.write_all(NEWLINE)?;
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

/// Configuration of formatting
pub struct Config<A: AddressFormatting, B: ByteFormatting, T: ByteFormatting> {
    fmt: Formatters<A, B, T>,

    bytes_per_row: usize,
    byte_grouping: usize,

    decorations: Decorations,
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
            decorations: Decorations { third_column_sep },
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
            decorations: Default::default(),
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

pub struct Decorations {
    third_column_sep: (String, String),
}

impl Default for Decorations {
    fn default() -> Self {
        Self { third_column_sep: ("|".to_string(), "|".to_string()) }
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
            let s = fmt.format(&self.buf);
            let _ = out.write_all(s.as_bytes())?;

            self.avail = 0;
        }

        Ok(len)
    }

    fn flush<T: ByteFormatting>(&mut self, out: &mut dyn Write, fmt: &mut T) -> Result<()> {
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
                Ok(chr) => {
                    if chr.is_ascii_printable() && !chr.is_ascii_control() {
                        chr.to_string()
                    } else {
                        ".".to_string()
                    }
                }
                Err(_) => ".".to_string(),
            })
            .collect();
        strs.join("")
    }

    fn padding_string(&mut self, byte_count: usize) -> String {
        " ".repeat(byte_count)
    }
}

pub struct StrictPrinter<
    O: Write,
    A: AddressFormatting,
    B: ByteFormatting,
    T: ByteFormatting,
    const GROUP: usize,
> {
    out: Option<Printer<O, A, B, T>>,

    buf: Vec<u8>,
    avail: usize,
}

impl<O: Write, A: AddressFormatting, B: ByteFormatting, T: ByteFormatting, const GROUP: usize>
    StrictPrinter<O, A, B, T, GROUP>
{
    pub fn new(out: O, addr: usize, config: StrictConfig<A, B, T>) -> Self {
        let n_groups = match GROUP {
            0 => 1,
            _ => {
                if config.number_of_groups > 0 {
                    config.number_of_groups
                } else {
                    DEFAULT_NUMBER_OF_GROUPS
                }
            }
        };
        let group_size = if GROUP > 0 {
            GROUP
        } else {
            DEFAULT_BYTES_PER_ROW
        };

        let config = Config::new(
            config.fmt,
            n_groups * group_size,
            group_size,
            config.decorations.third_column_sep,
        );
        let bytes_per_row = config.bytes_per_row;
        let out = Printer::new(out, addr as usize, config);

        Self {
            out: Some(out),
            buf: vec![0u8; bytes_per_row],
            avail: 0,
        }
    }

    pub fn finish(mut self) -> O {
        let mut out = self.out.take().unwrap();
        _ = out.write_all(&mut self.buf[..self.avail]);
        out.finish()
    }
}

impl<O: Write, A: AddressFormatting, B: ByteFormatting, T: ByteFormatting, const GROUP: usize> Write
    for StrictPrinter<O, A, B, T, GROUP>
{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        use std::io::*;
        match self.write_groupped(buf) {
            Ok(written) => Ok(written),
            Err(e) => Err(Error::new(ErrorKind::Other, e.to_string())),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        use std::io::*;
        let mut out = self.out.take().unwrap();

        match self.try_write_all_available() {
            Ok(_) => (),
            Err(e) => {
                self.out = Some(out);
                return Err(Error::new(ErrorKind::Other, e.to_string()));
            }
        }

        let result = out.flush();
        self.out = Some(out);
        result
    }
}

impl<O: Write, A: AddressFormatting, B: ByteFormatting, T: ByteFormatting, const GROUP: usize>
    StrictPrinter<O, A, B, T, GROUP>
{
    fn write_groupped(&mut self, buf: &[u8]) -> Result<usize> {
        let mut tmp = buf;
        while tmp.len() > 0 {
            let fill_count = min(self.buf.len() - self.avail, tmp.len());

            tmp.read_exact(&mut self.buf[self.avail..self.avail + fill_count])?;

            self.avail += fill_count;
            if self.avail == self.buf.len() {
                self.try_write_all_available()?;
            }
        }

        Ok(buf.len())
    }

    fn try_write_all_available(&mut self) -> Result<()> {
        if self.avail != self.buf.len() {
            return Err(Error::new(
                ErrorKind::Other,
                "StrictPrinter: Buffer is not fulfilled",
            ));
        }

        let mut out = self.out.take().unwrap();
        let result = out.write_all(&mut self.buf);
        self.out = Some(out);

        self.avail = 0;

        result
    }
}

impl<O: Write, A: AddressFormatting + Default, B: ByteFormatting + Default, T: ByteFormatting + Default, const GROUP: usize>
    StrictPrinter<O, A, B, T, GROUP>
{
    pub fn default_with(out: O, start_address: usize) -> StrictPrinter<O, A, B, T, GROUP> {
        Self::new(out, start_address, StrictConfig::<A, B, T>::default())
    }
}

impl<O: Write, const GROUP: usize> StrictPrinter<O, AddressFormatter, ByteFormatter, CharFormatter, GROUP> {
    pub fn default_fmt_with(
        out: O,
        start_address: usize,
    ) -> StrictPrinter<O, AddressFormatter, ByteFormatter, CharFormatter, GROUP> {
        Self::new(
            out,
            start_address,
            StrictConfig::<AddressFormatter, ByteFormatter, CharFormatter>::default(),
        )
    }
}

/// Configuration of strict formatting
pub struct StrictConfig<A: AddressFormatting, B: ByteFormatting, T: ByteFormatting> {
    fmt: Formatters<A, B, T>,
    number_of_groups: usize,
    decorations: Decorations,
}

impl<A: AddressFormatting, B: ByteFormatting, T: ByteFormatting> StrictConfig<A, B, T> {
    /// Create new config.
    /// `number_of_groups` should be greater than zero, otherwise it defaults to [`DEFAULT_NUMBER_OF_GROUPS`]
    pub fn new(
        fmt: Formatters<A, B, T>,
        number_of_groups: usize,
        decorations: Decorations,
    ) -> Self {
        Self {
            fmt,
            number_of_groups,
            decorations,
        }
    }
}

impl<A: AddressFormatting + Default, B: ByteFormatting + Default, T: ByteFormatting + Default>
    Default for StrictConfig<A, B, T>
{
    fn default() -> Self {
        let fmt: Formatters<A, B, T> = Formatters::new(A::default(), B::default(), T::default());
        Self {
            fmt,
            number_of_groups: DEFAULT_NUMBER_OF_GROUPS,
            decorations: Default::default(),
        }
    }
}
