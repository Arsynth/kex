use super::result::*;
use getopts::*;
use kex::{AddressStyle, ByteStyle, Group, Groupping};

use super::AppError;

/// ### Examples
///
/// -a h8 - 00021aa0
///
/// -a b16 - 0001001011010101
///
/// -a d8 - 09254854
///
/// -a o8 - 07556577
pub(super) const ADDR_FORMAT_SHORT_NAME: &str = "a";

/// ### Examples
///
/// -b h - AF B2 AA E0
///
/// -b b - 00010111 00010001 00011011 01010111
///
/// -b d - 135 255 127 100
///
/// -b o - 377 122 303 222
///
/// -b c - . y . w
///
/// -b C - ^@  y  ^A  w
pub(super) const BYTE_FORMAT_SHORT_NAME: &str = "b";

/// -g 2/4 - ab ac  ad ae  af b0  af b1
/// -g 8 - ab ac ad ae af b0 af b1
pub(super) const GROUPPING_SHORT_NAME: &str = "g";

pub(super) const SKIP_SHORT_NAME: &str = "s";
pub(super) const N_BYTES_SHORT_NAME: &str = "n";

const DEF_GROUP_SIZE: usize = 8;
const DEF_NGROUPS: usize = 2;

pub(super) fn get_configured_opts() -> Options {
    let mut opts = Options::new();

    opts.optopt(
        ADDR_FORMAT_SHORT_NAME,
        "",
        "-a h|b|d|o[min_width]",
        "address_format",
    );

    opts.optopt(
        BYTE_FORMAT_SHORT_NAME,
        "",
        &format!("-b h|b|d|o|c|C\nh - hexadecimal\nb - binary\nd - decimal\no - octal\nc - ASCII characters\ncaret notation with ASCII characters"),
        "byte_format",
    );

    opts.optopt(
        GROUPPING_SHORT_NAME,
        "",
        "-g 4/4\n",
        "group_size[/num_of_groups]",
    );

    opts.optopt(
        SKIP_SHORT_NAME,
        "",
        "Number of bytes to skip",
        "POSITIVE_INTEGER",
    );

    opts.optopt(
        N_BYTES_SHORT_NAME,
        "",
        "Number of bytes to read",
        "POSITIVE_INTEGER",
    );

    opts
}

pub(super) trait FromMatches {
    fn new(matches: &Matches) -> AppResult<Self>
    where
        Self: Sized;
}

impl FromMatches for AddressStyle {
    fn new(matches: &Matches) -> AppResult<Self>
    where
        Self: Sized,
    {
        let fmt_str = match matches.opt_get_default(ADDR_FORMAT_SHORT_NAME, "h8".to_string()) {
            Ok(s) => s,
            Err(e) => {
                return Err(AppError::new(format!("{e}")));
            }
        };

        Self::from_arg_str(fmt_str)
    }
}

trait FromArgStr {
    fn from_arg_str(fmt_str: String) -> AppResult<Self>
    where
        Self: Sized;
}

impl FromArgStr for AddressStyle {
    fn from_arg_str(fmt_str: String) -> AppResult<Self> {
        let mut fmt_chars = fmt_str.chars();
        let fmt_name = fmt_chars
            .next()
            .expect("That's not possible to have an empty argument");

        let rem = String::from_iter(fmt_chars);
        let min_width = if rem.len() != 0 {
            match rem.parse::<usize>() {
                Ok(i) => i,
                Err(e) => {
                    return Err(AppError::new(format!("{e}")));
                }
            }
        } else {
            8
        };

        match fmt_name {
            'h' => Ok(AddressStyle::Hex(min_width)),
            'b' => Ok(AddressStyle::Bin(min_width)),
            'd' => Ok(AddressStyle::Dec(min_width)),
            'o' => Ok(AddressStyle::Oct(min_width)),
            _ => {
                return Err(AppError::new(format!("{fmt_name}: Unknown address format")));
            }
        }
    }
}

impl FromMatches for ByteStyle {
    fn new(matches: &Matches) -> AppResult<Self>
    where
        Self: Sized,
    {
        let fmt_str = match matches.opt_get_default(BYTE_FORMAT_SHORT_NAME, "h".to_string()) {
            Ok(s) => s,
            Err(e) => {
                return Err(AppError::new(format!("{e}")));
            }
        };

        Self::from_arg_str(fmt_str)
    }
}

impl FromArgStr for ByteStyle {
    fn from_arg_str(fmt_str: String) -> AppResult<Self>
    where
        Self: Sized,
    {
        let mut fmt_chars = fmt_str.chars();
        let fmt_name = fmt_chars
            .next()
            .expect("That's not possible to have an empty argument");

        match fmt_name {
            'h' => Ok(Self::Hex),
            'b' => Ok(Self::Bin),
            'd' => Ok(Self::Dec),
            'o' => Ok(Self::Oct),
            'c' => Ok(Self::Ascii),
            'C' => Ok(Self::CaretAscii),
            _ => {
                return Err(AppError::new(format!("{fmt_name}: Unknown address format")));
            }
        }
    }
}

impl FromMatches for Groupping {
    fn new(matches: &Matches) -> AppResult<Self>
    where
        Self: Sized,
    {
        let fmt_str = match matches.opt_get_default(
            GROUPPING_SHORT_NAME,
            format!("{DEF_GROUP_SIZE}/{DEF_NGROUPS}"),
        ) {
            Ok(s) => s,
            Err(e) => {
                return Err(AppError::new(format!("{e}")));
            }
        };

        Self::from_arg_str(fmt_str)
    }
}

impl FromArgStr for Groupping {
    fn from_arg_str(fmt_str: String) -> AppResult<Self>
    where
        Self: Sized,
    {
        const DELIMITER_CHAR: char = '/';

        if fmt_str.contains(DELIMITER_CHAR) {
            let mut split = fmt_str.split(DELIMITER_CHAR);
            let group_size = split
                .next()
                .expect("That's not possible to have an empty argument");

            let group_size = match group_size {
                "" => DEF_GROUP_SIZE,
                _ => match group_size.parse::<usize>() {
                    Ok(i) => i,
                    Err(e) => {
                        return Err(AppError::new(format!("{e}")));
                    }
                },
            };

            let n_groups = split.next();
            let n_groups = match n_groups {
                Some(n) => match n {
                    "" => DEF_NGROUPS,
                    _ => match n.parse::<usize>() {
                        Ok(i) => i,
                        Err(e) => {
                            return Err(AppError::new(format!("{e}")));
                        }
                    },
                },
                None => DEF_NGROUPS,
            };

            Ok(Groupping::RepeatingGroup(
                Group::new(group_size, "  "),
                n_groups,
            ))
        } else {
            match fmt_str.parse::<usize>() {
                Ok(i) => Ok(Groupping::RowWide(i)),
                Err(e) => {
                    return Err(AppError::new(format!("{e}")));
                }
            }
        }
    }
}

#[derive(Clone)]
pub(crate) struct ContentRange {
    pub(crate) skip: usize,
    pub(crate) len: Option<usize>,
}

impl FromMatches for ContentRange {
    fn new(matches: &Matches) -> AppResult<Self>
    where
        Self: Sized,
    {
        let skip = match matches.opt_get_default(SKIP_SHORT_NAME, 0) {
            Ok(s) => s,
            Err(e) => {
                return Err(AppError::new(format!("{e}")));
            }
        };

        let len: Option<usize> = match matches.opt_get(N_BYTES_SHORT_NAME) {
            Ok(val) => val,
            Err(e) => {
                return Err(AppError::new(format!("{e}")));
            }
        };

        Ok(Self { skip, len })
    }
}
