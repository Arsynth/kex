use std::{fmt::Display};

use super::*;

/// Builtin byte formatter (used for `third` column by default)
#[derive(Clone)]
pub struct CharFormatter {
    placeholder: Vec<u8>,
    pub(super) separators: Separators,
}

impl CharFormatter {
    pub fn new(placeholder: impl Display, separators: Separators) -> Self {
        let placeholder = Vec::from(placeholder.to_string().as_bytes());

        Self {
            placeholder,
            separators,
        }
    }
}

impl CharFormatting for CharFormatter {
    fn format<O: Write>(&mut self, bytes: &[u8], out: &mut O) -> Result<usize> {
        let placeholder = &self.placeholder[..];

        for i in 0..bytes.len() {
            match AsciiChar::from_ascii(bytes[i]) {
                Ok(chr) => {
                    if chr.is_ascii_printable() && !chr.is_ascii_control() {
                        out.write_all(&bytes[i..i + 1])?;
                    } else {
                        out.write_all(placeholder)?;
                    }
                }
                Err(_) => {
                    out.write_all(placeholder)?;
                }
            }
        }

        Ok(bytes.len())
    }

    fn format_padding<O: Write>(&mut self, byte_count: usize, out: &mut O) -> Result<()> {
        out.write_all(&b" ".repeat(byte_count))
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
