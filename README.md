# kex
Streamed hex dumping library.

# Features
* Streamed I/O.
* Works with output, implementing `Write` trait.
* Customizable formatting

# Customized formatting
```
00000000🤩 deadbe efdead beefde 💥.........💨
00000009🤩 adbeef deadbe efdead 💥.........💨
00000012🤩 beefde adbeef deadbe 💥.........💨
0000001b🤩 efdead beefde adbeef 💥.........💨
00000024🤩 deadbe ef4974 20776f 💥....It wo💨
0000002d🤩 726b73 212121 feedfa 💥rks!!!...💨
00000036🤩 feedfa feedfa feedfa 💥.........💨
0000003f🤩 feedfa feedfa feedfa 💥.........💨
00000048🤩 feedfa feedfa feedfa 💥.........💨
00000051🤩 feedfa XxXxXx XxXxXx 💥...      💨
```

# Examples

```rust
use kex::*;

fn main() {
    use std::{
        fs::File,
        io::{stdout, Stdout},
    };

    let fmt = Formatters::new(
        MyAddrFormatter::new(),
        MyByteFormatter::new(),
        CharFormatter::new(),
    );
    let config = Config::new(fmt, 9, 3, ('\u{1F4A5}'.to_string(), '\u{1F4A8}'.to_string()));
    let mut printer = Printer::new(Box::new(stdout()), 0, config);
    
    let mut _printer =
        Printer::<Box<Stdout>, AddressFormatter, ByteFormatter, CharFormatter>::default_with(
            Box::new(stdout()),
            0,
        );

    let mut _printer = Printer::default_fmt_with(Box::new(stdout()), 0);

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

    println!("\nPrinting to vector:\n");

    let out = Box::new(Vec::<u8>::new());
    let mut printer = Printer::default_fmt_with(out, 0);

    _ = printer.push(bytes1);
    _ = printer.push(it_works);
    _ = printer.push(bytes2);

    let out = printer.finish();

    let result = std::str::from_utf8(&*out).unwrap();
    println!("{}", result);

    let file = File::create("target/hexdump.txt").unwrap();
    let mut printer = Printer::default_fmt_with(file, 0);
    _ = printer.push(bytes1);
    _ = printer.push(it_works);
    _ = printer.push(bytes2);
    _ = printer.finish();
}

struct MyAddrFormatter {
    fmt: AddressFormatter,
}

impl MyAddrFormatter {
    fn new() -> Self {
        MyAddrFormatter {
            fmt: AddressFormatter::new(8),
        }
    }
}

impl AddressFormatting for MyAddrFormatter {
    fn format(&self, addr: usize) -> String {
        const EMOJI: char = '\u{1F929}';

        format!("{}{EMOJI}", self.fmt.format(addr))
    }
}

struct MyByteFormatter {
    fmt: ByteFormatter,
}

impl MyByteFormatter {
    fn new() -> Self {
        Self {
            fmt: ByteFormatter::new(),
        }
    }
}

impl ByteFormatting for MyByteFormatter {
    fn format(&mut self, bytes: &[u8]) -> String {
        self.fmt.format(bytes)
    }
    
    fn padding_string(&mut self, byte_count: usize) -> String {
        format!("Xx").repeat(byte_count)
    }
}
```
