use std::io::*;

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

    fn format<O: Write>(&self, bytes: &[u8], byte_number_in_row: usize, out: &mut O) -> Result<usize> {
        let gr = &self.groupping;

        let sep = gr.separator();

        let mut tmp = bytes;

        let mut byte_number = byte_number_in_row;
        while tmp.len() != 0 {
            let to_format = min(tmp.len(), gr.bytes_left_in_group_after(byte_number));

            let needs_separator = byte_number != 0 && gr.is_aligned_at(byte_number);
            if needs_separator {
                out.write_all(&sep[..])?;
            }

            byte_number += to_format;

            if to_format != 0 {
                if self.is_little_endian {
                    for byte in tmp[..to_format].iter().rev() {
                        Self::format_byte(*byte, out)?;
                    }
                } else {
                    for byte in &tmp[..to_format] {
                        Self::format_byte(*byte, out)?;
                    }
                }
            }

            tmp = &tmp[to_format..]
        }

        Ok(bytes.len())
    }

    fn format_padding<O: Write>(&self, byte_number_in_row: usize, out: &mut O) -> Result<()> {
        let padding = b"..";
        let gr = &self.groupping;
        let sep = gr.separator();

        let mut tmp = gr.bytes_per_row() - byte_number_in_row;
        let mut byte_number = byte_number_in_row;
        while tmp != 0 {
            let to_format = min(tmp, gr.bytes_left_in_group_after(byte_number));

            let needs_separator = byte_number != 0 && gr.is_aligned_at(byte_number);
            if needs_separator {
                out.write_all(&sep)?;
            }

            byte_number += to_format;

            if to_format != 0 {
                out.write_all(&padding.repeat(to_format))?;
            }

            tmp -= to_format;
        }

        Ok(())
    }

    fn separators(&self) -> &Separators {
        &self.separators
    }
}

const LOWER_HEX: [u8; 16] = [
    b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'a', b'b', b'c', b'd', b'e', b'f',
];

impl ByteFormatter {
    #[inline(always)]
    fn format_byte<O: Write>(byte: u8, out: &mut O) -> Result<()> {
        let mut buf: [u8; 2] = [0, 0];
        buf[0] = LOWER_HEX[(byte >> 4) as usize];
        buf[1] = LOWER_HEX[(byte & 0x0f) as usize];

        out.write_all(&buf)?;
        
        Ok(())
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
            const WRITE_ERROR_MSG: &str = "Formatting error";

            let mut out = Vec::<u8>::new();
            let mut num = 0usize;
            for part in self.parts.iter() {
                fmt.format(&part, num, &mut out).expect(WRITE_ERROR_MSG);
                num += part.len();
            }

            fmt.format_padding(num, &mut out).unwrap();

            let out = String::from_utf8(out).expect(WRITE_ERROR_MSG);

            assert_eq!(out, self.result);
        }
    }

    #[test]
    fn test_unordered() {
        let fmt = ByteFormatter::new(
            Groupping::RepeatingGroup(Group::new(4, ""), 1),
            false,
            Default::default(),
        );
        let cases = vec![Case::new(
            vec![vec![0xfeu8, 0xed, 0xfa], vec![0xce]],
            "feedface",
        )];
        for case in cases {
            case.run(&fmt);
        }

        let fmt = ByteFormatter::new(
            Groupping::RepeatingGroup(Group::new(4, " "), 2),
            false,
            Default::default(),
        );
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
    }

    #[test]
    fn test_little_endian() {
        let fmt = ByteFormatter::new(
            Groupping::RepeatingGroup(Group::new(4, "-"), 4),
            true,
            Default::default(),
        );
        let cases = vec![
            Case::new(
                vec![vec![
                    0xfeu8, 0xed, 0xfa, 0xce, 0xfe, 0xed, 0xfa, 0xce, 0xfe, 0xed, 0xfa, 0xce, 0xfe,
                    0xed, 0xfa, 0xce,
                ]],
                "cefaedfe-cefaedfe-cefaedfe-cefaedfe",
            ),
            Case::new(
                vec![vec![0xfeu8, 0xed, 0xfa, 0xce, 0xfe, 0xed, 0xfa, 0xce]],
                "cefaedfe-cefaedfe-........-........",
            ),
            Case::new(
                vec![vec![
                    0xfeu8, 0xed, 0xfa, 0xce, 0xfe, 0xed, 0xfa, 0xce, 0xfe, 0xed,
                ]],
                "cefaedfe-cefaedfe-edfe....-........",
            ),
        ];
        for case in cases {
            case.run(&fmt);
        }
    }
}
