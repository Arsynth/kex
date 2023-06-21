use getopts::*;
use kex::*;
use std::env;
use std::{
    fs::File,
    io::{stdin, stdout, Read, Stdin, Stdout, Write},
};

mod result;
pub(crate) use result::*;

mod opts;
use opts::*;

pub(crate) fn get_app_config() -> AppResult<AppConfig> {
    let args = env::args().skip(1);

    let opts = get_configured_opts();

    let matches = match opts.parse(args) {
        Ok(m) => m,
        Err(e) => {
            return Err(AppError::new(format!("{e}")));
        }
    };

    AppConfig::new(matches)
}

pub(crate) struct AppConfig {
    pub(crate) input: Input,
    pub(crate) output: Output,
}

impl AppConfig {
    fn new(matches: Matches) -> AppResult<Self> {
        let input = Input::new(&matches)?;
        let output = Output::new(&matches)?;

        Ok(Self { input, output })
    }
}

pub(crate) enum Input {
    File(File),
    Stdin(Stdin),
}

impl Input {
    fn new(matches: &Matches) -> AppResult<Self> {
        let free_args = &matches.free;

        if free_args.len() != 0 {
            let file = File::open(free_args[0].clone())?;
            Ok(Input::File(file))
        } else {
            Ok(Input::Stdin(stdin()))
        }
    }
}

impl Read for Input {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Input::File(f) => f.read(buf),
            Input::Stdin(i) => i.lock().read(buf),
        }
    }
}

pub(crate) struct Output {
    printer: Printer<Stdout, AddressFormatter, ByteFormatter, CharFormatter>,
}

impl Output {
    fn new(matches: &Matches) -> AppResult<Self> {
        let byte_style = ByteStyle::new(matches)?;
        let char_formatter = match byte_style {
            ByteStyle::Ascii | ByteStyle::CaretAscii => None,
            _ => Some(CharFormatter::new(".", Separators::new(" |", "|")))
        };

        let config = Config::new(
            Some(AddressFormatter::new(
                AddressStyle::new(matches)?,
                Separators::new("", " "),
            )),
            ByteFormatter::new(
                byte_style,
                Groupping::RepeatingGroup(Group::new(8, "  "), 2),
                " ",
                false,
                Separators::new(" ", " "),
            ),
            char_formatter,
            true,
        );

        Ok(Self {
            printer: Printer::new(stdout(), 0, config),
        })
    }
}

impl Write for Output {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.printer.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.printer.flush()
    }
}

impl From<std::io::Error> for AppError {
    fn from(value: std::io::Error) -> Self {
        AppError::new(format!("{value}"))
    }
}