use std::{
    fs::File,
    io::{Read, Write, stdin, Seek}, path::Path, process::exit,
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

    match input.content {
        Content::Files(files) => {
            for path in files {
                handle_file_path(&path, &mut output, input.range.clone())
            }
        }
        Content::Stdin => handle_stdin(output, input.range),
    }
}

fn handle_stdin(output: impl Write, range: ContentRange) {
    let input = stdin().lock();
    handle(input, output, range.len)
}

fn handle_file_path(path: &str, output: &mut impl Write, range: ContentRange) {
    use std::io::SeekFrom;

    match File::open(Path::new(path)) {
        Ok(mut file) => {
            match file.seek(SeekFrom::Start(range.skip as u64)) {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("{e}");
                    exit(1);
                },
            }
            handle(&mut file, output, range.len)
        },
        Err(err) => {
            eprintln!("{path}: {err}");
            return;
        }
    }
}

fn handle(mut input: impl Read, mut output: impl Write, n_bytes: Option<usize>) {
    use std::cmp::min;

    let mut buf = [0u8; 4096];
    
    let mut elapsed = 0;
    let mut to_read = buf.len();

    while to_read != 0 {
        if let Some(n_bytes) = n_bytes {
            let diff = n_bytes - min(n_bytes, elapsed);
            to_read = min(to_read, diff);
        }

        if let Ok(size) = input.read(&mut buf[..to_read]) {
            if size == 0 {
                break;
            }
            assert!(output.write_all(&mut buf[..size]).is_ok());

            elapsed += size;
        } else {
            break;
        }

    }
}
