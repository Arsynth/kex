//! Example of endianess

use std::{io::Write};

use kex::*;

const GROUP_SIZE: usize = 4;
const NUM_OF_GROUPS: usize = 4;

fn main() {
    let data = "Lorem ipsum dolor sit amet".as_bytes();
    
    println!("Printing in big endian");
    print_data_per_byte(data, false);
    println!("");
    
    println!("Printing in little endian");
    print_data_per_byte(data, true);
    println!("");
}

fn print_data_per_byte(data: &[u8], is_little_endian: bool) {
    use std::io::stdout;

    let config = Config::new(
        Some(AddressFormatter::new(AddressStyle::Hex(16), Default::default())),
        ByteFormatter::new(
            Groupping::RepeatingGroup(Group::new(GROUP_SIZE, " - "), NUM_OF_GROUPS),
            "/",
            is_little_endian,
            Default::default()
        ),
        Some(CharFormatter::default()),
        false,
    );

    let mut printer = Printer::new(Box::new(stdout()), 0 as usize, config);

    
    for s in data {
        assert!(printer
            .write(&[*s])
            .is_ok());
    }

    _ = printer.finish();
    println!("");
}