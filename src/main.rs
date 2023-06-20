use kex::*;
use std::fs::File;
use std::io::stdout;
use std::io::{Read, Write};

fn main() {

    let mut buf = [0u8; 4096];

    // let stdin = std::io::stdin();
    // let mut handle = stdin.lock();

    let config = Config::new(
        Some(AddressFormatter::new(AddressStyle::Hex(8), Separators::new("", " "))),
        ByteFormatter::new(Groupping::RepeatingGroup(Group::new(8, "  "), 2), " ", false, Separators::new(" ", " ")),
        Some(CharFormatter::new(".", Separators::new(" |", "|"))),
        true,
    );
    let mut printer = Printer::new(stdout(), 0, config);
    
    let mut file = File::open("/bin/cat").expect("Can't open file");

    while let Ok(size) = file.read(&mut buf) {
        if size == 0 {
            break;
        }
        assert!(printer.write_all(&mut buf[..size]).is_ok());
    }

    printer.finish();
}