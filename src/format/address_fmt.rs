
use super::*;

/// Builtin address formatter
#[derive(Clone)]
pub struct AddressFormatter {
    pub style: AddressStyle,
    pub separators: Separators,
}

impl AddressFormatter {
    pub fn new(style: AddressStyle, separators: Separators) -> AddressFormatter {
        Self {
            style,
            separators,
        }
    }
}

impl Default for AddressFormatter {
    fn default() -> Self {
        Self { style: Default::default(), separators: Default::default() }
    }
}

impl AddressFormatting for AddressFormatter {
    fn format(&self, addr: usize) -> String {
        match self.style {
            AddressStyle::Dec(w) => {
                format!("{:width$}", addr, width = w)
            },
            AddressStyle::Hex(w) => {
                format!("{:0width$x}", addr, width = w)
            },
        }
    }

    fn separators(&self) -> &Separators {
        &self.separators
    }
}

#[derive(Clone)]
pub enum AddressStyle {
    /// Present address as decimal with specified minimum width. Formated address
    /// will be padded with spaces
    Dec(usize),
    /// Present address as hexadecimal with specified minimum width. Formated address
    /// will be padded with zeros
    Hex(usize),
}

impl Default for AddressStyle {
    fn default() -> Self {
        Self::Hex(8)
    }
}