# kex
Streamed hex dumping library.

# Features
* Streamed I/O.
* Works with output, implementing `Write` trait.
* Customizable formatting

# Examples

```rust
fn main() {
    use kex::*;
    use std::{io::{stdout, Stdout}, fs::File};

    let fmt = Formatters::new(
        AddressFormatter::new(8),
        ByteFormatter::new(),
        CharFormatter::new(),
    );
    let config = Config::new(fmt, 9, 3, ("<".to_string(), ">".to_string()));
    let mut printer = Printer::new(Box::new(stdout()), 0, config);
    let mut _printer = Printer::<Box<Stdout>, AddressFormatter, ByteFormatter, CharFormatter>::default_with(Box::new(stdout()), 0);
    
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
```
