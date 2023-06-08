
use ascii::*;

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

pub struct Decorations {
    pub(super) third_column_sep: (String, String),
}

impl Default for Decorations {
    fn default() -> Self {
        Self { third_column_sep: ("|".to_string(), "|".to_string()) }
    }
}


/// Used for address formatting (`first` column)
pub trait AddressFormatting {
    fn format(&self, addr: usize) -> String;
}

/// Used for bytes formatting (both for `second` and `third` columns)
pub trait ByteFormatting {
    fn format(&mut self, bytes: &[u8]) -> String;

    /// For the flexibility purpose (for example, you may need add ANSI color codes to output data),
    /// there are no strict checking for printable byte format length.
    /// Getting the spacing string with incorrect length will result with inaccurate output
    fn padding_string(&mut self, byte_count: usize) -> String;
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

/// Builtin byte formatter (used for `second` column by default)
pub struct ByteFormatter {}

impl ByteFormatter {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for ByteFormatter {
    fn default() -> Self {
        Self {}
    }
}

impl ByteFormatting for ByteFormatter {
    fn format(&mut self, bytes: &[u8]) -> String {
        let strs: Vec<String> = bytes.iter().map(|b| format!("{:02x}", b)).collect();
        strs.join("")
    }

    fn padding_string(&mut self, byte_count: usize) -> String {
        "..".repeat(byte_count)
    }
}

/// Builtin byte formatter (used for `third` column by default)
pub struct CharFormatter {}

impl CharFormatter {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for CharFormatter {
    fn default() -> Self {
        Self {}
    }
}

impl ByteFormatting for CharFormatter {
    fn format(&mut self, bytes: &[u8]) -> String {
        let strs: Vec<String> = bytes
            .iter()
            .map(|b| match AsciiChar::from_ascii(*b) {
                Ok(chr) => {
                    if chr.is_ascii_printable() && !chr.is_ascii_control() {
                        chr.to_string()
                    } else {
                        ".".to_string()
                    }
                }
                Err(_) => ".".to_string(),
            })
            .collect();
        strs.join("")
    }

    fn padding_string(&mut self, byte_count: usize) -> String {
        " ".repeat(byte_count)
    }
}