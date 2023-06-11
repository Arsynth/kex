// //! Example of customized printing format
// use kex::*;

// fn main() {
//     use std::io::stdout;

//     let fmt = Formatters::new(
//         MyAddrFormatter::new(),
//         MyByteFormatter::new(),
//         CharFormatter::default(),
//     );
//     let config = Config::new(
//         fmt,
//         Decorations::new(('\u{1F4A5}'.to_string(), '\u{1F4A8}'.to_string())),
//     );
//     let mut printer = Printer::new(Box::new(stdout()), 0, config);

//     let bytes1 = &[222u8, 173, 190, 239];
//     let bytes2 = &[0xfeu8, 0xed, 0xfa];
//     let it_works = &[
//         0x49u8, 0x74, 0x20, 0x77, 0x6f, 0x72, 0x6b, 0x73, 0x21, 0x21, 0x21,
//     ];

//     for _ in 0..10 {
//         _ = printer.push(bytes1);
//     }

//     _ = printer.push(it_works);

//     for _ in 0..11 {
//         _ = printer.push(bytes2);
//     }

//     printer.finish();
// }

// struct MyAddrFormatter {
//     fmt: AddressFormatter,
// }

// impl MyAddrFormatter {
//     fn new() -> Self {
//         MyAddrFormatter {
//             fmt: AddressFormatter::new(8),
//         }
//     }
// }

// impl AddressFormatting for MyAddrFormatter {
//     fn format(&self, addr: usize) -> String {
//         const EMOJI: char = '\u{1F929}';

//         format!("{}{EMOJI}", self.fmt.format(addr))
//     }
// }

// struct MyByteFormatter {
//     fmt: ByteFormatter,
// }

// impl MyByteFormatter {
//     fn new() -> Self {
//         Self {
//             fmt: ByteFormatter::new(),
//         }
//     }
// }

// impl ByteFormatting for MyByteFormatter {
//     fn format(&mut self, bytes: &[u8]) -> String {
//         self.fmt.format(bytes)
//     }

//     fn padding_string(&mut self, byte_count: usize) -> String {
//         format!("Xx").repeat(byte_count)
//     }
// }

fn main() {
    
}