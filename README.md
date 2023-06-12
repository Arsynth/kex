# kex

Streamed hex dumping library.

# Features
* Streamed I/O. [See the demo in asciinema](https://asciinema.org/a/591057)
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
https://docs.rs/kex/0.1.8/kex/

# Customized formatting
```
       0🤩deadbeef#deadbeef#deadbeef#deadbeef 💥................💨
      16🤩deadbeef#deadbeef#deadbeef#deadbeef 💥................💨
      32🤩deadbeef#deadbeef#49742077#6f726b73 💥........It works💨
      48🤩212121fe#edfafeed#fafeedfa#feedfafe 💥!!!.............💨
      64🤩edfafeed#fafeedfa#feedfafe#edfafeed 💥................💨
      80🤩fafeedfa#........#........#........ 💥....            💨
```

# Bug reports or feature requests
https://github.com/Arsynth/kex/issues