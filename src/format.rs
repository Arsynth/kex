use ascii::*;

pub const DEFAULT_BYTES_PER_ROW: usize = 16;
pub const DEFAULT_GROUP_SIZE: usize = 4;
pub const DEFAULT_NUMBER_OF_GROUPS: usize = 4;

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

pub trait ByteOrdered {
    /// Requirement for byte portions passing in the `format(...)` function
    fn byte_order(&self) -> ByteOrder;

    fn groupping(&self) -> &Groupping;
}

/// Requirement for providing byte portions
pub enum ByteOrder {
    /// Bytes will be provided in portions strictly according to grouping.
    /// In that case, buffering will be used
    Strict,
    /// Bytes will be provided as soon as new data available in [`super::Printer`]
    Relaxed,
}

/// Byte formatting style
pub enum Groupping {
    /// Single group with bytes count
    RowWide(usize),
    /// Group with repeat count. `bytes per row` = `group size` * `number of groups`
    RepeatingGroup(Group, usize),
    /// Number of bytes in row does not depend on group size, it is specified directly.
    ///
    /// ## Note
    /// If the bytes per row is not aligned to group size, last group will be incomplete sized
    BytesPerRow(usize, Group),
}

impl Groupping {
    fn separator(&self) -> String {
        match self {
            Groupping::RowWide(_) => "".to_string(),
            Groupping::RepeatingGroup(g, _) => g.separator.clone(),
            Groupping::BytesPerRow(_, g) => g.separator.clone(),
        }
    }

    fn is_aligned_at(&self, number: usize, len: usize) -> bool {
        let bpr = self.bytes_per_row();
        assert!(
            bpr >= number + len,
            "is_aligned_at(): Trying to exceed maximum row length"
        );

        match self {
            Groupping::RowWide(_) => number == 0 && len == bpr,
            Groupping::RepeatingGroup(g, _) | Groupping::BytesPerRow(_, g) => {
                let rem = bpr % g.size;
                if rem == 0 {
                    number % g.size == 0 && len == g.size
                } else {
                    let rem_group = rem;
                    let n_aligned_groups = (bpr - rem_group) / g.size;

                    number == n_aligned_groups * g.size && len == rem_group
                }
            }
        }
    }

    fn bytes_per_row(&self) -> usize {
        match self {
            Groupping::RowWide(r) => *r,
            Groupping::RepeatingGroup(g, rep) => g.size * rep,
            Groupping::BytesPerRow(r, _) => *r,
        }
    }

    fn number_of_groups(&self) -> usize {
        match self {
            Groupping::RowWide(_) => 1,
            Groupping::RepeatingGroup(_, rep) => *rep,
            Groupping::BytesPerRow(r, g) => {
                let rem = r % g.size;
                if rem == 0 {
                    r / g.size
                } else {
                    (r - rem) / g.size + 1
                }
            }
        }
    }

    fn group_of_byte(&self, number: usize) -> usize {
        assert!(
            self.bytes_per_row() >= number,
            "group_of_byte():Trying to exceed maximum row length"
        );
        match self {
            Groupping::RowWide(_) => 0,
            Groupping::RepeatingGroup(g, _) | Groupping::BytesPerRow(_, g) => {
                let group_size = g.size;
                let rem = number % group_size;
                (number - rem) / group_size
            }
        }
    }

    fn bytes_left_in_group_after(&self, number: usize) -> usize {
        assert!(
            self.bytes_per_row() >= number,
            "bytes_left_in_group_after(): Trying to exceed maximum row length"
        );
        match self {
            Groupping::RowWide(r) => r - number,
            Groupping::RepeatingGroup(g, _) | Groupping::BytesPerRow(_, g) => {
                let group_num = self.group_of_byte(number);
                g.size * group_num - number
            }
        }
    }
}

impl Default for Groupping {
    fn default() -> Self {
        Self::RepeatingGroup(Default::default(), DEFAULT_NUMBER_OF_GROUPS)
    }
}

pub struct Group {
    /// Number of bytes in the group
    pub(super) size: usize,
    pub(super) separator: String,
}

impl Group {
    pub fn new(size: usize, separator: &str) -> Self {
        Self {
            size,
            separator: separator.to_string(),
        }
    }
}

impl Default for Group {
    fn default() -> Self {
        Self::new(DEFAULT_GROUP_SIZE, " ")
    }
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
pub struct ByteFormatter {
    groupping: Groupping,
    is_little_endian: bool,
}

impl ByteFormatter {
    pub fn new(groupping: Groupping, is_little_endian: bool) -> Self {
        Self {
            groupping,
            is_little_endian,
        }
    }
}

impl Default for ByteFormatter {
    fn default() -> Self {
        Self {
            groupping: Default::default(),
            is_little_endian: false,
        }
    }
}

impl ByteFormatting for ByteFormatter {
    fn format(&mut self, bytes: &[u8], byte_number_in_row: usize) -> String {
        use std::cmp::min;

        let gr = &self.groupping;

        if let ByteOrder::Strict = self.byte_order() {
            assert!(
                gr.is_aligned_at(byte_number_in_row, bytes.len()),
                "ByteOrder::Strict require that provided bytes are aligned to groups"
            );
        }

        let mut result = String::new();

        let sep = gr.separator();
        let mut tmp = bytes;

        let mut byte_number = byte_number_in_row;
        while tmp.len() != 0 {
            let to_format = min(tmp.len(), gr.bytes_left_in_group_after(byte_number));

            byte_number += to_format;

            let needs_separator = tmp.len() > 0 && to_format == 0;
            if needs_separator {
                result += &sep;
            }

            if to_format != 0 {
                for byte in &tmp[..to_format] {
                    result += &format!("{:02x}", byte);
                }
            }

            tmp = &tmp[to_format..]
        }

        result
    }

    fn padding_string(&mut self, byte_count: usize, byte_number_in_row: usize) -> String {
        "..".repeat(byte_count)
    }
}

impl ByteOrdered for ByteFormatter {
    fn byte_order(&self) -> ByteOrder {
        if self.is_little_endian {
            ByteOrder::Strict
        } else {
            ByteOrder::Relaxed
        }
    }

    fn groupping(&self) -> &Groupping {
        &self.groupping
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

#[cfg(test)]
mod tests {
    use super::*;

    struct Case {
        parts: Vec<Vec<u8>>,
        bpr: usize,

        result: String,
    }

    impl Case {
        fn run(&self, fmt: &mut ByteFormatter) {
            let mut out = String::new();
            let mut num = 0usize;
            for part in self.parts.iter() {
                let s = fmt.format(&part[..], num);
                out += &s;

                num += part.len();
            }

            out += &fmt.padding_string(self.bpr - num, num);
        }
    }

    #[test]
    fn test_unordered() {
        let cases = [(vec![0xfeu8, 0xed, 0xfa, 0xce])];
        assert!(true);
    }

    #[test]
    fn test_little_endian() {
        assert!(false);
    }
}
