use std::io::Write;

use kex::*;

const GROUP_SIZE: usize = 4;

fn main() {
    use std::io::stdout;

    let fmt = Formatters::new(
        AddressFormatter::new(16),
        LittleEndianFormatter::new(),
        CharFormatter::new(),
    );
    let config = StrictConfig::new(fmt, 4, 4, Default::default());

    let mut printer = StrictGrouppedPrinter::new(Box::new(stdout()), 0 as usize, config);

    assert!(printer
        .write("Lorem ipsum dolor sit amet".as_bytes())
        .is_ok());

    _ = printer.finish();
}

pub struct LittleEndianFormatter {}

impl LittleEndianFormatter {
    pub fn new() -> Self {
        Self {}
    }
}

impl ByteFormatting for LittleEndianFormatter {
    fn format(&mut self, bytes: &[u8]) -> String {
        assert!(bytes.len() <= GROUP_SIZE);

        let strs: Vec<String> = bytes
            .iter()
            .rev()
            .map(|b: &u8| format!("{:02x}", b))
            .collect();
        strs.join("")
    }

    fn padding_string(&mut self, byte_count: usize) -> String {
        "..".repeat(byte_count)
    }
}
