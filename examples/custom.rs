//! Example of customized printing format
use kex::*;

fn main() {
    use std::io::stdout;

    let config = Config::new(
        Some(MyAddrFormatter::new()),
        MyByteFormatter::new(),
        Some(CharFormatter::new(
            ".".to_string(),
            Separators::new(&'\u{1F4A5}'.to_string(), &'\u{1F4A8}'.to_string()),
        )),
    );
    let mut printer = Printer::new(Box::new(stdout()), 0, config);

    let bytes1 = &[222u8, 173, 190, 239];
    let bytes2 = &[0xfeu8, 0xed, 0xfa];
    let it_works = &[
        0x49u8, 0x74, 0x20, 0x77, 0x6f, 0x72, 0x6b, 0x73, 0x21, 0x21, 0x21,
    ];

    for _ in 0..10 {
        _ = printer.push(bytes1);
    }

    _ = printer.push(it_works);

    for _ in 0..11 {
        _ = printer.push(bytes2);
    }

    printer.finish();
}

#[derive(Clone)]
struct MyAddrFormatter {
    fmt: AddressFormatter,
}

impl MyAddrFormatter {
    fn new() -> Self {
        MyAddrFormatter {
            fmt: AddressFormatter::new(8, Separators::new("", &'\u{1F929}'.to_string())),
        }
    }
}

impl AddressFormatting for MyAddrFormatter {
    fn format(&self, addr: usize) -> String {
        self.fmt.format(addr)
    }

    fn separators(&self) -> &Separators {
        self.fmt.separators()
    }
}

#[derive(Clone)]
struct MyByteFormatter {
    fmt: ByteFormatter,
}

impl MyByteFormatter {
    fn new() -> Self {
        Self {
            fmt: ByteFormatter::new(
                Groupping::RepeatingGroup(Group::new(4, "#"), 4),
                false,
                Default::default(),
            ),
        }
    }
}

impl ByteFormatting for MyByteFormatter {
    fn byte_order(&self) -> ByteOrder {
        self.fmt.byte_order()
    }

    fn groupping(&self) -> Groupping {
        self.fmt.groupping()
    }

    fn format(&self, bytes: &[u8], byte_number_in_row: usize) -> String {
        self.fmt.format(bytes, byte_number_in_row)
    }

    fn padding_string(&self, byte_number_in_row: usize) -> String {
        self.fmt.padding_string(byte_number_in_row)
    }

    fn separators(&self) -> &Separators {
        self.fmt.separators()
    }
}
