use ascii::*;
use std::cmp::min;

pub mod byte;
pub use byte::*;

pub mod groupping;
pub use groupping::*;

/// Formatters for address (first column), bytes (second column), and text (third column)
pub struct Formatters<A: AddressFormatting, B: ByteFormatting, T: ByteFormatting> {
    pub(super) addr: A,
    pub(super) byte: B,
    pub(super) text: T,
}

impl<A: AddressFormatting, B: ByteFormatting, T: ByteFormatting> Formatters<A, B, T> {
    pub fn new(addr: A, byte: B, text: T) -> Self {
        Self { addr, byte, text }
    }
}

/// Used for address formatting (`first` column)
pub trait AddressFormatting {
    fn format(&self, addr: usize) -> String;
}

/// Used for bytes formatting (both for `second` and `third` columns)
pub trait ByteFormatting {
    /// `bytes` - bytes to convert to `String`
    ///
    /// `byte_number_in_row` - number of byte in row (from where the `bytes` started formatting).
    /// It useful for determining, where to place group separators (if your formatter uses it)
    fn format(&mut self, bytes: &[u8], byte_number_in_row: usize) -> String;

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
    fn padding_string(&mut self, byte_count: usize, byte_number_in_row: usize) -> String;
}

/// Builtin address formatter
pub struct AddressFormatter {
    min_width: usize,
}

impl AddressFormatter {
    pub fn new(min_width: usize) -> AddressFormatter {
        Self { min_width }
    }
}

impl Default for AddressFormatter {
    fn default() -> Self {
        Self { min_width: 8 }
    }
}

impl AddressFormatting for AddressFormatter {
    fn format(&self, addr: usize) -> String {
        format!("{:0width$x}", addr, width = self.min_width)
    }
}

/// Builtin byte formatter (used for `third` column by default)
pub struct CharFormatter {
    placeholder: String,
}

impl CharFormatter {
    pub fn new(placeholder: String) -> Self {
        Self { placeholder }
    }
}

impl Default for CharFormatter {
    fn default() -> Self {
        Self::new(".".to_string())
    }
}

impl ByteFormatting for CharFormatter {
    fn format(&mut self, bytes: &[u8], _byte_number_in_row: usize) -> String {
        let placeholder = &self.placeholder;
        let strs: Vec<String> = bytes
            .iter()
            .map(|b| match AsciiChar::from_ascii(*b) {
                Ok(chr) => {
                    if chr.is_ascii_printable() && !chr.is_ascii_control() {
                        chr.to_string()
                    } else {
                        placeholder.clone()
                    }
                }
                Err(_) => placeholder.clone(),
            })
            .collect();
        strs.join("")
    }

    fn padding_string(&mut self, byte_count: usize, _byte_number_in_row: usize) -> String {
        " ".repeat(byte_count)
    }
}
