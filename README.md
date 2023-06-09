# kex

Streamed hex dumping library.

# Features
* Streamed I/O.
* Works with output, implementing `Write` trait.
* Customizable formatting

# Examples
## One of examples
```rust
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
    
    let mut printer = Printer::default_fmt_with(Box::new(stdout()), 0);

    while let Ok(size) = handle.read(&mut buf) {
        if size == 0 {
            break;
        }
        assert!(printer.write_all(&mut buf[..size]).is_ok());
    }

    printer.finish();
}
```


See all the examples in `examples` directory in the crate root

# Documentation
https://docs.rs/kex/0.1.7/kex/

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