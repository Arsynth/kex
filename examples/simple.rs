//! Simple example
use kex::*;

fn main() {
    use std::io::stdout;

    let mut printer = Printer::default_fmt_with(stdout(), 0);

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

    printer.finish();
}