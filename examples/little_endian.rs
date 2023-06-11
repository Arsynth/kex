//! Example of strict groupping

use std::io::Write;

use kex::*;

const GROUP_SIZE: usize = 4;
const NUM_OF_GROUPS: usize = 4;

fn main() {
    use std::io::stdout;

    let fmt = Formatters::new(
        AddressFormatter::new(16),
        ByteFormatter::new(
            Groupping::RepeatingGroup(Group::new(GROUP_SIZE, "-"), NUM_OF_GROUPS),
            true,
        ),
        CharFormatter::default(),
    );
    let config = Config::new(fmt, Decorations::default());

    let mut printer = Printer::new(Box::new(stdout()), 0 as usize, config);

    assert!(printer
        .write("Lorem ipsum dolor sit amet".as_bytes())
        .is_ok());

    _ = printer.finish();
}
