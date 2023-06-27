//! Module with builtin raw bytes formatter

use std::io::*;

use super::*;

const PLACEHOLDER: &[u8; 1] = b".";
const SPACE: u8 = b' ';
const CARET: u8 = b'^';

/// Builtin byte formatter (used for `second` column by default)
#[derive(Clone)]
pub struct ByteFormatter {
    pub(super) style: ByteStyle,
    pub(super) groupping: Groupping,
    pub(super) is_little_endian: bool,

    byte_separator: Vec<u8>,
    pub(super) separators: Separators,
}

impl ByteFormatter {
    pub fn new(
        style: ByteStyle,
        groupping: Groupping,
        byte_separator: &str,
        is_little_endian: bool,
        separators: Separators,
    ) -> Self {
        Self {
            style,
            groupping,
            is_little_endian,
            byte_separator: Vec::from(byte_separator),
            separators,
        }
    }
}

impl Default for ByteFormatter {
    fn default() -> Self {
        Self {
            style: Default::default(),
            groupping: Default::default(),
            is_little_endian: false,
            byte_separator: vec![],
            separators: Default::default(),
        }
    }
}

impl ByteFormatting for ByteFormatter {
    fn byte_order(&self) -> GroupAtomicity {
        if self.is_little_endian {
            GroupAtomicity::Required
        } else {
            GroupAtomicity::Optional
        }
    }

    fn groupping(&self) -> Groupping {
        self.groupping.clone()
    }

    fn format<O: Write>(
        &self,
        bytes: &[u8],
        byte_number_in_row: usize,
        out: &mut O,
    ) -> Result<usize> {
        let gr = &self.groupping;
        let gr_size = gr.max_group_size();

        let sep = gr.separator();

        let mut tmp = bytes;

        let mut byte_number = byte_number_in_row;
        while tmp.len() != 0 {
            let bytes_left_in_group = gr.bytes_left_in_group_after(byte_number);
            let to_format = min(tmp.len(), bytes_left_in_group);

            let needs_separator = byte_number != 0 && gr.is_aligned_at(byte_number);
            if needs_separator {
                out.write_all(&sep[..])?;
            }

            byte_number += to_format;

            if to_format != 0 {
                let mut num = gr_size - bytes_left_in_group;

                if self.is_little_endian {
                    for byte in tmp[..to_format].iter().rev() {
                        if num != 0 {
                            out.write_all(&self.byte_separator[..])?;
                        }
                        self.style.format_byte(*byte, out)?;
                        num += 1;
                    }
                } else {
                    for byte in &tmp[..to_format] {
                        if num != 0 {
                            out.write_all(&self.byte_separator[..])?;
                        }
                        self.style.format_byte(*byte, out)?;
                        num += 1;
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
        let gr_size = gr.max_group_size();

        let sep = gr.separator();

        let mut tmp = gr.bytes_per_row() - byte_number_in_row;
        let mut byte_number = byte_number_in_row;
        while tmp != 0 {
            let bytes_left_in_group = gr.bytes_left_in_group_after(byte_number);
            let to_format = min(tmp, bytes_left_in_group);

            let needs_separator = byte_number != 0 && gr.is_aligned_at(byte_number);
            if needs_separator {
                out.write_all(&sep)?;
            }

            byte_number += to_format;

            if to_format != 0 {
                let mut num = gr_size - bytes_left_in_group;

                for _ in 0..to_format {
                    if num != 0 {
                        out.write_all(&self.byte_separator)?;
                    }
                    out.write_all(padding)?;

                    num += 1;
                }
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

const CARET_NOTATION_LUT: [u8; 32] = [
    b'@', b'A', b'B', b'C', b'D', b'E', b'F', b'G', b'H', b'I', b'J', b'K', b'L', b'M', b'N', b'O',
    b'P', b'Q', b'R', b'S', b'T', b'U', b'V', b'W', b'X', b'Y', b'Z', b'[', b'\\', b']', b'^',
    b'_',
];

const CARET_NOTATION_DEL: u8 = b'?';

#[derive(Clone)]
pub enum ByteStyle {
    Hex,
    Bin,
    Dec,
    Oct,
    Ascii,
    CaretAscii,
}

impl ByteStyle {
    #[inline(always)]
    pub(super) fn format_byte<O: Write>(&self, byte: u8, out: &mut O) -> Result<()> {
        match self {
            ByteStyle::Hex => self.fmt_hex(byte, out),
            ByteStyle::Bin => self.fmt_bin(byte, out),
            ByteStyle::Dec => self.fmt_dec(byte, out),
            ByteStyle::Oct => self.fmt_oct(byte, out),
            ByteStyle::Ascii => self.fmt_ascii(byte, out),
            ByteStyle::CaretAscii => self.fmt_caret_ascii(byte, out),
        }
    }

    pub(super) fn fmt_hex<O: Write>(&self, byte: u8, out: &mut O) -> Result<()> {
        let mut buf: [u8; 2] = [0, 0];
        buf[0] = LOWER_HEX[(byte >> 4) as usize];
        buf[1] = LOWER_HEX[(byte & 0x0f) as usize];

        out.write_all(&buf)?;

        Ok(())
    }

    pub(super) fn fmt_bin<O: Write>(&self, byte: u8, out: &mut O) -> Result<()> {
        let s = format!("{:08b}", byte);
        out.write_all(s.as_bytes())?;

        Ok(())
    }

    pub(super) fn fmt_dec<O: Write>(&self, byte: u8, out: &mut O) -> Result<()> {
        let s = format!("{:3}", byte);
        out.write_all(s.as_bytes())?;

        Ok(())
    }

    pub(super) fn fmt_oct<O: Write>(&self, byte: u8, out: &mut O) -> Result<()> {
        let s = format!("{:03o}", byte);
        out.write_all(s.as_bytes())?;

        Ok(())
    }

    pub(super) fn fmt_ascii<O: Write>(&self, byte: u8, out: &mut O) -> Result<()> {
        let chr = unsafe { byte.to_ascii_char_unchecked() };

        if chr.is_ascii_printable() {
            out.write_all(&vec![byte])?;
        } else {
            out.write(PLACEHOLDER)?;
        }

        Ok(())
    }

    pub(super) fn fmt_caret_ascii<O: Write>(&self, byte: u8, out: &mut O) -> Result<()> {
        let mut buf: [u8; 2] = [0, 0];

        let chr = unsafe { byte.to_ascii_char_unchecked() };

        if (byte as usize) < CARET_NOTATION_LUT.len() {
            buf[0] = CARET;
            buf[1] = CARET_NOTATION_LUT[byte as usize];
        } else if byte == b'?' {
            buf[0] = CARET;
            buf[1] = CARET_NOTATION_DEL;
        } else if chr.is_ascii_printable() {
            buf[0] = SPACE;
            buf[1] = byte;
        } else {
            buf[0] = SPACE;
            buf[1] = PLACEHOLDER[0];
        }

        out.write_all(&buf)?;

        Ok(())
    }
}

impl Default for ByteStyle {
    fn default() -> Self {
        Self::Hex
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
            Default::default(),
            Groupping::RepeatingGroup(Group::new(4, ""), 1),
            "",
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
            Default::default(),
            Groupping::RepeatingGroup(Group::new(4, " "), 2),
            "",
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
            Default::default(),
            Groupping::RepeatingGroup(Group::new(4, "-"), 4),
            "",
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
