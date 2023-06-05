use std::io::stdout;

use kex::*;

fn main() {
    let fmt = Formatters::new(
        AddressFormatter::new(8),
        ByteFormatter::new(),
        CharFormatter::new(),
    );

    let config = Config::new(fmt, 8, 3, ("<".to_string(), ">".to_string()));

    let mut printer = Printer::new(Box::new(stdout()), 0, config);

    let bytes1 = &[222u8, 173, 190, 239];
    let bytes2 = &[0xfeu8, 0xed, 0xfa];
    let it_works = &[
        0x49u8, 0x74, 0x20, 0x77, 0x6f, 0x72, 0x6b, 0x73, 0x21, 0x21, 0x21,
    ];

    // for _ in 0..10 {
    //     _ = printer.push(bytes1);
    // }

    _ = printer.push(it_works);

    for _ in 0..11 {
        _ = printer.push(bytes2);
    }
}
