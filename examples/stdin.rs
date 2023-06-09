//! Example for reading from `stdin`

use std::io::{Read, Write};

use kex::*;

/// Usage:
/// cargo run --example stdin
/// 
/// Or:
/// cat /bin/cat | cargo run --example stdin
/// 
fn main() {
    use std::io::stdout;

    let mut buf = [0u8; 64];
    let stdin = std::io::stdin();
    let mut handle = stdin.lock();
    
    let mut printer = Printer::default_fmt_with(stdout(), 0);

    while let Ok(size) = handle.read(&mut buf) {
        if size == 0 {
            break;
        }
        assert!(printer.write_all(&mut buf[..size]).is_ok());
    }

    printer.finish();
}