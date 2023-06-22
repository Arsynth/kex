use std::{
    fs::File,
    io::{Read, Write, stdin}, path::Path,
};

mod app;
use app::*;

fn main() {
    let app_config = match get_app_config() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    };

    let AppConfig {input, mut output} = app_config;

    match input {
        Input::Files(files) => {
            for path in files {
                handle_file(&path, &mut output)
            }
        }
        Input::Stdin => handle_stdin(output),
    }
}

fn handle_stdin(output: impl Write) {
    handle(stdin().lock(), output);
}

fn handle_file(path: &str, output: &mut impl Write) {
    match File::open(Path::new(path)) {
        Ok(file) => handle(file, output),
        Err(err) => {
            eprintln!("{path}: {err}");
            return;
        }
    }
}

fn handle(mut input: impl Read, mut output: impl Write) {
    let mut buf = [0u8; 4096];

    while let Ok(size) = input.read(&mut buf) {
        if size == 0 {
            break;
        }
        assert!(output.write_all(&mut buf[..size]).is_ok());
    }
}
