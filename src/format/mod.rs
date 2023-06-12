use ascii::*;
use std::cmp::min;

pub mod byte;
pub use byte::*;

pub mod ordering;
pub use ordering::*;

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
    fn format(&self, bytes: &[u8], byte_number_in_row: usize) -> String;

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
    fn padding_string(&self, byte_number_in_row: usize) -> String;

    fn separators(&self) -> &Separators;
}

pub trait CharFormatting {
    fn format(&mut self, bytes: &[u8]) -> String;
    fn padding_string(&mut self, byte_count: usize) -> String;

    fn separators(&self) -> &Separators;
}

/// Builtin address formatter
#[derive(Clone)]
pub struct AddressFormatter {
    min_width: usize,
    pub(super) separators: Separators,
}

impl AddressFormatter {
    pub fn new(min_width: usize, separators: Separators) -> AddressFormatter {
        Self {
            min_width,
            separators,
        }
    }
}

impl Default for AddressFormatter {
    fn default() -> Self {
        Self { min_width: 8, separators: Default::default() }
    }
}

impl AddressFormatting for AddressFormatter {
    fn format(&self, addr: usize) -> String {
        format!("{:0width$x}", addr, width = self.min_width)
    }

    fn separators(&self) -> &Separators {
        &self.separators
    }
}

/// Builtin byte formatter (used for `third` column by default)
#[derive(Clone)]
pub struct CharFormatter {
    placeholder: String,
    pub(super) separators: Separators,
}

impl CharFormatter {
    pub fn new(placeholder: String, separators: Separators) -> Self {
        Self {
            placeholder,
            separators,
        }
    }
}

impl CharFormatting for CharFormatter {
    fn format(&mut self, bytes: &[u8]) -> String {
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

    fn padding_string(&mut self, byte_count: usize) -> String {
        " ".repeat(byte_count)
    }

    fn separators(&self) -> &Separators {
        &self.separators
    }
}

impl Default for CharFormatter {
    fn default() -> Self {
        Self::new(".".to_string(), Separators::new("|", "|"))
    }
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
