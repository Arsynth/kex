
use super::*;

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