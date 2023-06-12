use super::*;

/// Builtin byte formatter (used for `second` column by default)
#[derive(Clone)]
pub struct ByteFormatter {
    pub(super) groupping: Groupping,
    pub(super) is_little_endian: bool,

    pub(super) separators: Separators,
}

impl ByteFormatter {
    pub fn new(groupping: Groupping, is_little_endian: bool, separators: Separators) -> Self {
        Self {
            groupping,
            is_little_endian,
            separators,
        }
    }
}

impl Default for ByteFormatter {
    fn default() -> Self {
        Self {
            groupping: Default::default(),
            is_little_endian: false,
            separators: Default::default(),
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

    fn groupping(&self) -> Groupping {
        self.groupping.clone()
    }

    fn format(&self, bytes: &[u8], byte_number_in_row: usize) -> String {
        let gr = &self.groupping;

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
                if self.is_little_endian {
                    for byte in tmp[..to_format].iter().rev() {
                        result += &format!("{:02x}", byte);
                    }
                } else {
                    for byte in &tmp[..to_format] {
                        result += &format!("{:02x}", byte);
                    }
                }
            }

            tmp = &tmp[to_format..]
        }

        result
    }

    fn padding_string(&self, byte_number_in_row: usize) -> String {
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

    fn separators(&self) -> &Separators {
        &self.separators
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

        fn run(&self, fmt: &ByteFormatter) {
            let mut out = String::new();
            let mut num = 0usize;
            for part in self.parts.iter() {
                let s = fmt.format(&part[..], num);
                out += &s;

                num += part.len();
            }

            out += &fmt.padding_string(num);

            assert_eq!(out, self.result);
        }
    }

    #[test]
    fn test_unordered() {
        let fmt = ByteFormatter::new(Groupping::RepeatingGroup(Group::new(4, ""), 1), false, Default::default());
        let cases = vec![Case::new(
            vec![vec![0xfeu8, 0xed, 0xfa], vec![0xce]],
            "feedface",
        )];
        for case in cases {
            case.run(&fmt);
        }

        let fmt = ByteFormatter::new(Groupping::RepeatingGroup(Group::new(4, " "), 2), false, Default::default());
        let cases = vec![
            Case::new(
                vec![vec![0xfeu8, 0xed, 0xfa], vec![0xce]],
                "feedface ........",
            ),
            Case::new(
                vec![vec![0xfeu8], vec![0xed, 0xfa, 0xce]],
                "feedface ........",
            ),
            Case::new(
                vec![vec![0xfeu8], vec![0xed], vec![0xfa, 0xce]],
                "feedface ........",
            ),
            Case::new(
                vec![vec![0xfeu8, 0xed, 0xfa], vec![0xce, 0xca]],
                "feedface ca......",
            ),
            Case::new(
                vec![vec![0xfeu8], vec![0xed, 0xfa, 0xce, 0xca]],
                "feedface ca......",
            ),
        ];

        for case in cases {
            case.run(&fmt);
        }

        let fmt = ByteFormatter::new(Groupping::RepeatingGroup(Group::new(4, "-"), 4), true, Default::default());
        let cases = vec![
            Case::new(
                vec![vec![
                    0xfeu8, 0xed, 0xfa, 0xce, 0xfe, 0xed, 0xfa, 0xce, 0xfe, 0xed, 0xfa, 0xce, 0xfe,
                    0xed, 0xfa, 0xce,
                ]],
                "cefaedfe-cefaedfe-cefaedfe-cefaedfe",
            ),
            Case::new(
                vec![vec![
                    0xfeu8, 0xed, 0xfa, 0xce, 0xfe, 0xed, 0xfa, 0xce
                ]],
                "cefaedfe-cefaedfe-........-........",
            ),
            Case::new(
                vec![vec![
                    0xfeu8, 0xed, 0xfa, 0xce, 0xfe, 0xed, 0xfa, 0xce, 0xfe, 0xed
                ]],
                "cefaedfe-cefaedfe-edfe....-........",
            ),
        ];
        for case in cases {
            case.run(&fmt);
        }
    }

    #[test]
    fn test_little_endian() {
        assert!(false);
    }
}