//! Example of customized printing format
use kex::*;

fn main() {
    use std::io::stdout;

    let config = Config::new(
        Some(AddressFormatter::new(
            AddressStyle::Dec(8),
            Separators::new("", &'\u{1F929}'.to_string()),
        )),
        ByteFormatter::new(
            Groupping::RepeatingGroup(Group::new(4, "#"), 4),
            false,
            Default::default(),
        ),
        Some(CharFormatter::new(
            ".".to_string(),
            Separators::new(&'\u{1F4A5}'.to_string(), &'\u{1F4A8}'.to_string()),
        )),
        true
    );
    let mut printer = Printer::new(Box::new(stdout()), 0, config);

    let bytes1 = &[222u8, 173, 190, 239];
    let bytes2 = &[0xfeu8, 0xed, 0xfa];
    let title = b"Custom printing";

    for _ in 0..10 {
        _ = printer.push(bytes1);
    }

    _ = printer.push(title);

    for _ in 0..11 {
        _ = printer.push(bytes2);
    }

    printer.finish();
}
