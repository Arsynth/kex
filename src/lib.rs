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
            streamer: Streamer::new(
                config.addr,
                config.byte,
                config.text,
                start_address,
                config.dedup_enabled,
            ),
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

    /// Does nothing. Always returns `Ok(())`
    fn flush(&mut self) -> Result<()> {
        Ok(())
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn simple() {
        let result = vec![];
        let mut printer = Printer::default_fmt_with(result, 0);

        let bytes1 = &[222u8, 173, 190, 239];
        let bytes2 = &[0xfeu8, 0xed, 0xfa];
        let title = b"Simple printing";

        for _ in 0..10 {
            _ = printer.push(bytes1);
        }

        _ = printer.push(title);

        for _ in 0..11 {
            _ = printer.push(bytes2);
        }

        let result = printer.finish();
        let result_str = String::from_utf8(result).expect("Invalid characters in result");

        let expected = "00000000 deadbeef deadbeef deadbeef deadbeef |................|
*
00000020 deadbeef deadbeef 53696d70 6c652070 |........Simple p|
00000030 72696e74 696e67fe edfafeed fafeedfa |rinting.........|
00000040 feedfafe edfafeed fafeedfa feedfafe |................|
00000050 edfafeed fafeedfa ........ ........ |........        |
00000058 \n";

        assert_eq!(result_str, expected);
    }

    #[test]
    fn stable_reading() {
        println!("Testing reading stability");
        stable_reading_with("testable/lorem_ipsum");
        stable_reading_with("testable/duplications");
    }
    
    fn stable_reading_with(path: &str) {
        println!("Path: {path}");

        let patterns = vec![
            vec![1],
            vec![1, 2, 1],
            vec![5],
            vec![4],
            vec![4, 7],
            vec![4, 1],
            vec![18, 1, 16, 7, 4, 5, 3],
            vec![2000],
            vec![444],
        ];
        let test_data =
            std::fs::read(path).expect("Could not opent testable data");
    
        let mut last_result: Option<String> = None;
    
        for pat in patterns {
            let result = string_with(&test_data, pat);
            if let Some(last_result) = last_result {
                assert_eq!(result, last_result);
            }
    
            last_result = Some(result);
        }
    }

    fn string_with(bytes: &[u8], read_len_pattern: Vec<usize>) -> String {
        use std::cmp::min;

        let result = vec![];
        let mut printer = Printer::default_fmt_with(result, 0);

        let mut tmp = bytes;

        let mut pat_idx = 0;
        while tmp.len() != 0 {
            let to_read = min(read_len_pattern[pat_idx], tmp.len());

            printer
                .write(&tmp[..to_read])
                .expect("Writing to printer error");

            tmp = &tmp[to_read..];

            pat_idx += 1;
            pat_idx %= read_len_pattern.len();
        }

        let result = printer.finish();
        String::from_utf8(result).expect("Invalid characters in result")
    }

    #[test]
    fn duplications() {
        let result = string_with_file("testable/duplications");
        let expected = "00000000 61626364 65666768 696a6b6c 6d6e6f70 |abcdefghijklmnop|
00000010 61626364 65666768 69316b6c 6d6e6f70 |abcdefghi1klmnop|
00000020 61626364 65666768 696a6b6c 6d6e6f70 |abcdefghijklmnop|
*
00000040 61626364 65666768 696a6b6c 6d6e6f67 |abcdefghijklmnog|
00000050 61626364 65666768 6935366c 6d6e6f70 |abcdefghi56lmnop|
00000060 61626364 65666768 696a6b6c 6d6e6f70 |abcdefghijklmnop|
*
000000e0 \n";

        assert_eq!(result, expected);
    }

    fn string_with_file(path: &str) -> String {
        let test_data =
            std::fs::read(path).expect("Could not opent testable data");
        string_with(&test_data, vec![100])
    }
}
