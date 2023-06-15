use std::{fmt::Display, io::Read};

use super::*;

/// Builtin byte formatter (used for `third` column by default)
#[derive(Clone)]
pub struct CharFormatter {
    placeholder: Vec<u8>,
    pub(super) separators: Separators,
}

impl CharFormatter {
    pub fn new(placeholder: impl Display, separators: Separators) -> Self {
        Self {
            placeholder: Vec::from(placeholder.to_string().as_bytes()),
            separators,
        }
    }
}

impl CharFormatting for CharFormatter {
    fn format(&mut self, bytes: &[u8]) -> String {
        use std::str::from_utf8_unchecked;

        let placeholder = &self.placeholder[..];
        let placeholder_len = placeholder.len();

        let mut result = vec![0u8; bytes.len() * self.placeholder.len()];
        let mut result_len = 0;

        for i in 0..bytes.len() {
            let mut placeholder = &placeholder[..];
            match AsciiChar::from_ascii(bytes[i]) {
                Ok(chr) => {
                    if chr.is_ascii_printable() && !chr.is_ascii_control() {
                        result[result_len] = bytes[i];
                        result_len += 1;
                    } else {
                        _ = placeholder.read_exact(&mut result[result_len..result_len + placeholder.len()]);
                        result_len += placeholder_len;
                    }
                }
                Err(_) => {
                    _ = placeholder.read_exact(&mut result[result_len..result_len + placeholder.len()]);
                    result_len += placeholder_len;
                }
            }
        }

        unsafe { from_utf8_unchecked(&result[..result_len]) }.to_string()
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
