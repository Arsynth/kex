use std::io::{Read, Write};

mod app;
use app::*;

fn main() {
    let mut app_config = match get_app_config() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        },
    };

    let mut buf = [0u8; 4096];

    while let Ok(size) = app_config.input.read(&mut buf) {
        if size == 0 {
            break;
        }
        assert!(app_config.output.write_all(&mut buf[..size]).is_ok());
    }
}
