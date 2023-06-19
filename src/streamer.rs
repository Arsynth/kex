use super::format::*;
use std::io::*;

const ROW_SEPARATOR: &[u8] = b"\n";
const DUPLICATE_PLACEHOLDER: &[u8] = b"*";

pub(super) struct Streamer<A: AddressFormatting, B: ByteFormatting, C: CharFormatting> {
    addr_fmt: Option<A>,
    byte_fmt: B,
    char_fmt: Option<C>,

    total_written: usize,
    printable_offset: usize,

    cache: Vec<u8>,
    available: usize,

    dedup_enabled: bool,
    row_state: RowState,
}

impl<A: AddressFormatting, B: ByteFormatting, C: CharFormatting> Streamer<A, B, C> {
    pub(super) fn new(
        addr_fmt: Option<A>,
        byte_fmt: B,
        char_fmt: Option<C>,
        printable_offset: usize,
        dedup_enabled: bool,
    ) -> Self {
        let bpr = byte_fmt.groupping().bytes_per_row();
        Self {
            addr_fmt,
            byte_fmt,
            char_fmt,
            total_written: 0,
            printable_offset,
            cache: vec![0u8; bpr],
            available: 0,
            dedup_enabled,
            row_state: RowState::CanWrite,
        }
    }

    pub(crate) fn push<O: std::io::Write>(&mut self, bytes: &[u8], out: &mut O) -> Result<()> {
        if self.dedup_enabled {
            self.push_deduplicated(bytes, out)
        } else {
            self.push_groupped(bytes, out)
        }
    }

    fn push_groupped<O: std::io::Write>(&mut self, bytes: &[u8], out: &mut O) -> Result<()> {
        use std::cmp::min;
        let mut tmp = &bytes[..];

        let gr = &self.byte_fmt.groupping();
        let bpr = gr.bytes_per_row();
        let group_size = gr.max_group_size();

        while tmp.len() != 0 {
            let byte_in_row = self.total_written % bpr;

            if self.available == 0 {
                self.start_row(out)?;
            }

            let to_cache = min(self.cache.len() - self.available, tmp.len());

            let old_available = self.available;
            if to_cache != 0 {
                tmp.read_exact(&mut self.cache[old_available..old_available + to_cache])?;
                self.available += to_cache;
            }

            assert!(self.available <= bpr, "Too much bytes written");

            let group_cache = self.calculated_group_cache(old_available, self.available);
            assert_eq!(group_cache.len() % group_size, 0, "Unaligned group cache");

            // Start reading from cache
            if group_cache.len() != 0 {
                self.total_written += self.byte_fmt.format(group_cache, byte_in_row, out)?;
            }

            // Finish row
            if self.available == bpr {
                self.finish_row(out)?;
            }
        }

        Ok(())
    }

    fn push_deduplicated<O: std::io::Write>(&mut self, bytes: &[u8], out: &mut O) -> Result<()> {
        use std::cmp::min;
        let mut tmp = &bytes[..];

        let gr = &self.byte_fmt.groupping();
        let bpr = gr.bytes_per_row();

        let mut row_was_written = false;

        while tmp.len() != 0 {
            match self.row_state {
                RowState::CanWrite => {
                    self.row_state = RowState::NeedsPlaceholder;
                }
                _ => (),
            }

            let ignore_dedup = self.total_written < bpr;

            let to_check = min(self.cache.len() - self.available, tmp.len());

            let cache_part = &self.cache[self.available..self.available + to_check];
            let should_write = ignore_dedup || &tmp[..to_check] != cache_part;

            row_was_written |= should_write;

            let mut cache_part = &mut self.cache[self.available..self.available + to_check];
            if should_write {
                tmp.read_exact(&mut cache_part)?;
            } else {
                tmp = &tmp[to_check..];
            }

            // Deduplication was previously interrupted and we continue write row
            self.available += to_check;

            if self.cache.len() - self.available == 0 {
                if row_was_written {
                    self.start_row(out)?;
                    self.total_written += self.byte_fmt.format(&self.cache, 0, out)?;
                    self.row_state = RowState::CanWrite;
                } else {
                    self.total_written += self.cache.len();
                }

                self.finish_row(out)?;

                row_was_written = false;
            }
        }

        Ok(())
    }

    pub(crate) fn write_tail<O: Write>(&mut self, out: &mut O) -> Result<()> {
        if self.dedup_enabled {
            self.write_current_offset(out)?;
        }

        if self.available == 0 {
            out.write_all(ROW_SEPARATOR)?;
            return Ok(());
        }

        let written_in_row = self.total_written % self.byte_fmt.groupping().bytes_per_row();
        assert!(
            self.available >= written_in_row,
            "Bytes written more than available"
        );

        let remaining = &self.cache[written_in_row..self.available];
        self.total_written += self.byte_fmt.format(remaining, written_in_row, out)?;

        self.finish_row(out)?;

        self.write_current_offset(out)?;

        out.write_all(ROW_SEPARATOR)?;

        Ok(())
    }

    #[inline(always)]
    fn calculated_group_cache(&self, at_number: usize, available: usize) -> &[u8] {
        assert!(
            at_number <= available,
            "Unbalanced groupping. Start number greater than end number"
        );

        let gr = &self.byte_fmt.groupping();
        let group_size = gr.max_group_size();

        let gr = self.byte_fmt.groupping();

        let start = gr.group_of_byte(at_number) * group_size;
        let end = gr.group_of_byte(available) * group_size;

        &self.cache[start..end]
    }

    fn start_row<O: Write>(&self, out: &mut O) -> Result<()> {
        self.write_current_offset(out)?;
        out.write_all(&self.byte_fmt.separators().trailing)?;

        Ok(())
    }

    fn write_current_offset<O: Write>(&self, out: &mut O) -> Result<()> {
        if let Some(fmt) = &self.addr_fmt {
            out.write_all(&fmt.separators().trailing)?;
            fmt.format(self.total_written + self.printable_offset, out)?;
            out.write_all(&fmt.separators().leading)?;
        }

        Ok(())
    }

    fn finish_row<O: Write>(&mut self, out: &mut O) -> Result<()> {
        match self.row_state {
            RowState::CanWrite => {
                self.byte_fmt.format_padding(self.available, out)?;

                out.write_all(&self.byte_fmt.separators().leading)?;

                self.write_text(out)?;

                out.write_all(ROW_SEPARATOR)?;
            }
            RowState::NeedsPlaceholder => {
                self.replace_row_with_placeholder(out)?;
            }
            RowState::Skipped => (),
        }

        self.available = 0;
        Ok(())
    }

    fn replace_row_with_placeholder<O: Write>(&mut self, out: &mut O) -> Result<()> {
        match self.row_state {
            RowState::NeedsPlaceholder => {
                self.row_state = RowState::Skipped;

                out.write_all(DUPLICATE_PLACEHOLDER)?;
                out.write_all(ROW_SEPARATOR)
            }
            _ => panic!("replace_row_with_placeholder(): Row does not need a placeholder"),
        }
    }

    fn write_text<O: Write>(&self, out: &mut O) -> Result<()> {
        if let Some(fmt) = &self.char_fmt {
            out.write_all(&fmt.separators().trailing)?;

            fmt.format(&self.cache[..self.available], out)?;

            let tail_len = self.byte_fmt.groupping().bytes_per_row() - self.available;
            fmt.format_padding(tail_len, out)?;

            out.write_all(&fmt.separators().leading)?;
        }

        Ok(())
    }
}

enum RowState {
    CanWrite,
    NeedsPlaceholder,
    Skipped,
}
