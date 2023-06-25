# kex
![](https://github.com/Arsynth/kex/actions/workflows/ci.yml/badge.svg)

Streamed hex dumping library.

# Features
* Streamed I/O.
* Works with output, implementing `Write` trait.
* Customizable formatting
* Row deduplication
* Very fast

# Demo
[![demo](https://asciinema.org/a/592589.svg)](https://asciinema.org/a/592589?autoplay=1)

# Binary
## Short guide

kex -a h8 -b h -g 8/2 file1 file2 file3 ...

Will print specified files sequentally as single stream. If no files specified, program will expect input from `stdin`

### Arguments

-a `<format>[min_width]` - the address format

`<format>` - address representation. Accepts values:
* h - hexadecimal
* b - binary
* d - decimal
* o - octal

`<min_width>` - minimum width of displayed address. All, but decimal will be padded by zeros. Decimal padded with empty spaces

-b `<format>` - format of the bytes (second column)

`<format>` - byte representation. Accepts values:
* h - hexadecimal
* b - binary
* d - decimal
* o - octal
* c - ASCII characters. (Excludes characters/third column)
* C - Caret notation + ASCII characters. (Excludes characters/third column)

-g `group_size[/number_of_groups]`

or

-g `bytes_per_row`


# Library

## Examples
### One of examples
```rust
use kex::*;
use std::fs::File;
use std::io::stdout;
use std::io::{Read, Write};

fn main() {

    let mut buf = [0u8; 4096];

    // let stdin = std::io::stdin();
    // let mut handle = stdin.lock();

    let config = Config::new(
        Some(AddressFormatter::new(AddressStyle::Hex(16), Default::default())),
        ByteFormatter::new(Groupping::RepeatingGroup(Group::new(4, " "), 4), false, Default::default()),
        Some(CharFormatter::default()),
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


```

See all the examples in `examples` directory in the crate root

## Documentation
https://docs.rs/kex/0.2.3/kex/

## Deduplication
```
0000000000000000 cafebabe 00000002 01000007 00000003 |................|
0000000000000010 00004000 000111c0 0000000e 0100000c |..@.............|
0000000000000020 80000002 00018000 0000d0f0 0000000e |................|
0000000000000030 00000000 00000000 00000000 00000000 |................|
*
0000000000004000 cffaedfe 07000001 03000000 02000000 |................|
0000000000004010 11000000 08060000 85002000 00000000 |.......... .....|
0000000000004020 19000000 48000000 5f5f5041 47455a45 |....H...__PAGEZE|
0000000000004030 524f0000 00000000 00000000 00000000 |RO..............|
```

## Customized formatting
```
       0🤩deadbeef#deadbeef#deadbeef#deadbeef 💥................💨
*
      32🤩deadbeef#deadbeef#43757374#6f6d2070 💥........Custom p💨
      48🤩72696e74#696e67fe#edfafeed#fafeedfa 💥rinting.........💨
      64🤩feedfafe#edfafeed#fafeedfa#feedfafe 💥................💨
      80🤩edfafeed#fafeedfa*
      88🤩
```

# Bug reports or feature requests
https://github.com/Arsynth/kex/issues