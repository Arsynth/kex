
use super::format::*;
use std::io::*;
use super::{Printer, config::*};
use std::cmp::min;

/// Same as [`Printer`], but with guarranty that `second column` formatter will receive portions
/// of bytes equal to `config.group_size` until `finish()` or `flush()` called.
/// After that calls, remainig bytes, which count lesser than `config.group_size`, will be sent into formatter,
pub struct StrictGrouppedPrinter<
    O: Write,
    A: AddressFormatting,
    B: ByteFormatting,
    T: ByteFormatting,
> {
    out: Option<Printer<O, A, B, T>>,

    buf: Vec<u8>,
    avail: usize,
}

impl<O: Write, A: AddressFormatting, B: ByteFormatting, T: ByteFormatting>
    StrictGrouppedPrinter<O, A, B, T>
{
    pub fn new(out: O, addr: usize, config: StrictConfig<A, B, T>) -> Self {
        let config = Config::new(
            config.fmt,
            config.number_of_groups * config.group_size,
            config.group_size,
            config.decorations,
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

impl<O: Write, A: AddressFormatting, B: ByteFormatting, T: ByteFormatting> Write
    for StrictGrouppedPrinter<O, A, B, T>
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

impl<O: Write, A: AddressFormatting, B: ByteFormatting, T: ByteFormatting>
    StrictGrouppedPrinter<O, A, B, T>
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

impl<O: Write, A: AddressFormatting + Default, B: ByteFormatting + Default, T: ByteFormatting + Default>
    StrictGrouppedPrinter<O, A, B, T>
{
    pub fn default_with(out: O, start_address: usize) -> StrictGrouppedPrinter<O, A, B, T> {
        Self::new(out, start_address, StrictConfig::<A, B, T>::default())
    }
}

impl<O: Write> StrictGrouppedPrinter<O, AddressFormatter, ByteFormatter, CharFormatter> {
    pub fn default_fmt_with(
        out: O,
        start_address: usize,
    ) -> StrictGrouppedPrinter<O, AddressFormatter, ByteFormatter, CharFormatter> {
        Self::new(
            out,
            start_address,
            StrictConfig::<AddressFormatter, ByteFormatter, CharFormatter>::default(),
        )
    }
}