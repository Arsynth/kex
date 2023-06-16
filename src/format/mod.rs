use ascii::*;
use std::{cmp::min, io::Write};

pub mod address_fmt;
pub use address_fmt::*;

pub mod byte_fmt;
pub use byte_fmt::*;

pub mod char_fmt;
pub use char_fmt::*;

pub mod ordering;
pub use ordering::*;

use std::io::Result;

/// Used for address formatting (`first` column)
pub trait AddressFormatting {
    fn format(&self, addr: usize) -> String;

    fn separators(&self) -> &Separators;
}

/// Used for bytes formatting (both for `second` and `third` columns)
pub trait ByteFormatting {
    /// Requirement for byte portions passing in the `format(...)` function
    fn byte_order(&self) -> ByteOrder;

    fn groupping(&self) -> Groupping;

    fn bytes_per_row(&self) -> usize {
        self.groupping().bytes_per_row()
    }

    /// `bytes` - bytes to convert to `String`
    ///
    /// `byte_number_in_row` - number of byte in row (from where the `bytes` started formatting).
    /// It useful for determining, where to place group separators (if your formatter uses it)
    fn format<O: Write>(&self, bytes: &[u8], byte_number_in_row: usize, out: &mut O) -> Result<usize>;

    /// When writing data chunks to [`super::Printer`] is finished, last output line may be incomplete.
    /// This function should provide spacing string for incomplete row
    ///
    /// ## Note
    /// For the flexibility purpose (for example, you may need add ANSI color codes to output data),
    /// there are no strict checking for printable byte format length.
    /// Getting the spacing string with incorrect length will result with inaccurate output
    ///
    /// ### Parameters
    /// `byte_count` - missing bytes, needed to complete row
    ///
    /// `byte_number_in_row` - number of byte in row (from where the `bytes` started formatting).
    /// It useful for determining, where to place group separators (if your formatter uses it)
    fn format_padding<O: Write>(&self, byte_number_in_row: usize, out: &mut O) -> Result<()>;

    fn separators(&self) -> &Separators;
}

pub trait CharFormatting {
    fn format<O: Write>(&mut self, bytes: &[u8], out: &mut O) -> Result<usize>;
    fn format_padding<O: Write>(&mut self, byte_count: usize, out: &mut O) -> Result<()>;

    fn separators(&self) -> &Separators;
}

#[derive(Clone)]
pub struct Separators {
    pub(crate) trailing: Vec<u8>,
    pub(crate) leaidng: Vec<u8>,
}

impl Separators {
    pub fn new(trailing: &str, leaidng: &str) -> Self {
        Self {
            leaidng: Vec::from(leaidng),
            trailing: Vec::from(trailing),
        }
    }
}

impl Default for Separators {
    fn default() -> Self {
        Self {
            leaidng: Vec::from(" "),
            trailing: Default::default(),
        }
    }
}
