
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
    fn format<O: Write>(&self, addr: usize, out: &mut O) -> Result<()> {
        let result = self.style.format(addr);
        out.write_all(result.as_bytes())
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
    /// Present address as binary with specified minimum width. Formated address
    /// will be padded with zeros
    Bin(usize),
    /// Present address as octal with specified minimum width. Formated address
    /// will be padded with zeros
    Oct(usize),
}

impl AddressStyle {
    fn format(&self, addr: usize) -> String {
        match self {
            AddressStyle::Dec(w) => format!("{:width$}", addr, width = w),
            AddressStyle::Hex(w) => format!("{:0width$x}", addr, width = w),
            AddressStyle::Bin(w) => format!("{:0width$b}", addr, width = w),
            AddressStyle::Oct(w) => format!("{:0width$o}", addr, width = w),
            
        }
    }
}

impl Default for AddressStyle {
    fn default() -> Self {
        Self::Hex(8)
    }
}