use super::*;

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