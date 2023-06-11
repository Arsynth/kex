use super::*;

/// Builtin byte formatter (used for `second` column by default)
pub struct ByteFormatter {
    pub(super) groupping: Groupping,
    pub(super) is_little_endian: bool,
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

    fn format(&self, bytes: &[u8], byte_number_in_row: usize) -> String {
        let gr = &self.groupping;

        if let ByteOrder::Strict = self.byte_order() {
            assert!(
                gr.is_aligned_range(byte_number_in_row, bytes.len()),
                "ByteOrder::Strict require that provided bytes are aligned to groups"
            );
        }

        let mut result = String::new();

        let sep = gr.separator();
        let mut tmp = bytes;

        let mut byte_number = byte_number_in_row;
        while tmp.len() != 0 {
            let to_format = min(tmp.len(), gr.bytes_left_in_group_after(byte_number));

            let needs_separator = byte_number != 0 && gr.is_aligned_at(byte_number);
            if needs_separator {
                result += &sep;
            }

            byte_number += to_format;

            if to_format != 0 {
                for byte in &tmp[..to_format] {
                    result += &format!("{:02x}", byte);
                }
            }

            tmp = &tmp[to_format..]
        }

        result
    }

    fn padding_string(&self, _byte_count: usize, byte_number_in_row: usize) -> String {
        let padding = "..";
        let gr = &self.groupping;
        let sep = gr.separator();

        let mut result = String::new();

        let mut tmp = gr.bytes_per_row() - byte_number_in_row;
        let mut byte_number = byte_number_in_row;
        while tmp != 0 {
            let to_format = min(tmp, gr.bytes_left_in_group_after(byte_number));

            
            let needs_separator = byte_number != 0 && gr.is_aligned_at(byte_number);
            if needs_separator {
                result += &sep;
            }
            
            byte_number += to_format;

            if to_format != 0 {
                result += &padding.repeat(to_format);
            }

            tmp -= to_format;
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Case {
        parts: Vec<Vec<u8>>,
        result: String,
    }

    impl Case {
        fn new(parts: Vec<Vec<u8>>, result: &str) -> Self {
            Self {
                parts,
                result: result.to_string(),
            }
        }

        fn run(&self, fmt: &mut ByteFormatter) {
            let mut out = String::new();
            let mut num = 0usize;
            for part in self.parts.iter() {
                let s = fmt.format(&part[..], num);
                out += &s;

                num += part.len();
            }

            out += &fmt.padding_string(fmt.groupping.bytes_per_row() - num, num);

            assert_eq!(out, self.result);
        }
    }

    #[test]
    fn test_unordered() {
        let mut fmt = ByteFormatter::new(Groupping::BytesPerRow(4, Group::new(4, "")), false);
        let cases = vec![
            Case::new(vec![vec![0xfeu8, 0xed, 0xfa], vec![0xce]], "feedface")
        ];
        for case in cases {
            case.run(&mut fmt);
        }
        
        let mut fmt = ByteFormatter::new(Groupping::BytesPerRow(8, Group::new(4, " ")), false);
        let cases = vec![
            Case::new(vec![vec![0xfeu8, 0xed, 0xfa], vec![0xce]], "feedface ........"),
            Case::new(vec![vec![0xfeu8], vec![0xed, 0xfa, 0xce]], "feedface ........"),
            Case::new(vec![vec![0xfeu8], vec![0xed], vec![0xfa, 0xce]], "feedface ........"),
            Case::new(vec![vec![0xfeu8, 0xed, 0xfa], vec![0xce, 0xca]], "feedface ca......"),
            Case::new(vec![vec![0xfeu8], vec![0xed, 0xfa, 0xce, 0xca]], "feedface ca......"),
            ];
        
        for case in cases {
            case.run(&mut fmt);
        }
    }

    #[test]
    fn test_little_endian() {
        assert!(false);
    }
}
