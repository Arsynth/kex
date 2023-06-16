use super::format::*;
use std::io::*;

const ROW_SEPARATOR: &[u8] = b"\n";

pub(super) struct Streamer<A: AddressFormatting, B: ByteFormatting, C: CharFormatting> {
    addr_fmt: Option<A>,
    byte_fmt: B,
    char_fmt: Option<C>,

    offset: usize,
    printable_offset: usize,

    cache: Vec<u8>,
    available: usize,
}

impl<A: AddressFormatting, B: ByteFormatting, C: CharFormatting> Streamer<A, B, C> {
    pub(super) fn new(
        addr_fmt: Option<A>,
        byte_fmt: B,
        char_fmt: Option<C>,
        printable_offset: usize,
    ) -> Self {
        let bpr = byte_fmt.groupping().bytes_per_row();
        Self {
            addr_fmt,
            byte_fmt,
            char_fmt,
            offset: 0,
            printable_offset,
            cache: vec![0u8; bpr],
            available: 0,
        }
    }

    pub(crate) fn push<O: std::io::Write>(&mut self, bytes: &[u8], out: &mut O) -> Result<()> {
        use std::cmp::min;
        let mut tmp = &bytes[..];

        let gr = &self.byte_fmt.groupping();
        let bpr = gr.bytes_per_row();
        let group_size = gr.max_group_size();

        while tmp.len() != 0 {
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
                self.offset += self.byte_fmt.format(group_cache, self.offset % bpr, out)?;
            }

            // Finish row
            if self.available == bpr {
                out.write_all(ROW_SEPARATOR)?;

                self.available = 0;
            }

            // tmp = &tmp[to_cache..];
        }

        Ok(())
    }

    pub(crate) fn write_tail<O: Write>(&self, out: &mut O) -> Result<()> {
        Ok(())
    }

    #[inline(always)]
    fn calculated_group_cache(&self, at_number: usize, available: usize) -> &[u8] {
        assert!(
            at_number <= available,
            "Unbalanced groupping. Start number greater than end number"
        );
        let group_size = self.byte_fmt.groupping().max_group_size();

        let gr = self.byte_fmt.groupping();

        let start = gr.group_of_byte(at_number) * group_size;
        let end = gr.group_of_byte(available) * group_size;

        &self.cache[start..end]
    }
}
