use super::format::*;
use std::cmp::min;
use std::io::*;

const LOST_BUFFER_MESSAGE: &str = "Somewhere we lost back buffer";

pub(super) struct Callbacks<SR: FnMut(), WR: FnMut(WriteResult), FR: FnMut()> {
    start_row_cb: SR,
    write_row_cb: WR,
    finish_row_cb: FR,
}

impl<SR, WR, FR> Callbacks<SR, WR, FR>
where
    SR: FnMut(),
    WR: FnMut(WriteResult),
    FR: FnMut(),
{
    pub(super) fn new(start_row_cb: SR, write_row_cb: WR, finish_row_cb: FR) -> Self {
        Self {
            start_row_cb,
            write_row_cb,
            finish_row_cb,
        }
    }
}

pub(super) struct GrouppedWriter {
    groupping: Groupping,
    order: ByteOrder,

    address: usize,

    back_buf: Option<Vec<u8>>,
    avail: usize,
}

impl GrouppedWriter {
    pub(super) fn address(&self) -> usize {
        self.address
    }
}

impl GrouppedWriter {
    pub(super) fn new(groupping: Groupping, order: ByteOrder) -> Self {
        let buf_size = match order {
            ByteOrder::Strict => groupping.max_group_size(),
            ByteOrder::Relaxed => 0,
        };
        Self {
            groupping,
            order,
            back_buf: Some(vec![0u8; buf_size]),
            address: 0,
            avail: 0,
        }
    }
}

impl GrouppedWriter {
    pub(super) fn write<SR, WR, FR>(
        &mut self,
        buf: &[u8],
        callbacks: &mut Callbacks<SR, WR, FR>,
    ) -> std::io::Result<usize>
    where
        SR: FnMut(),
        WR: FnMut(WriteResult),
        FR: FnMut(),
    {
        let mut back_buf = self.back_buf.take().expect(LOST_BUFFER_MESSAGE);

        let gr = &self.groupping;
        let bpr = gr.bytes_per_row();
        let byte_in_row = self.address % bpr;

        if byte_in_row == 0 {
            (callbacks.start_row_cb)();
        }

        let result = match self.order {
            ByteOrder::Strict => {
                assert!(
                    back_buf.len() > 0,
                    "Back buffer is zero length while ByteOrder::Strict"
                );

                let remaining = gr.bytes_left_in_group_after(byte_in_row);
                let byte_in_group = gr.byte_number_in_group(byte_in_row);
                let fill_count = min(remaining, buf.len());

                let mut buf_to_read = &mut back_buf[byte_in_group..byte_in_group + fill_count];
                let mut buf = &buf[..];
                buf.read_exact(&mut buf_to_read)
                    .expect("Could not read from buffer");

                let is_row_finished = byte_in_row + fill_count == bpr;

                if byte_in_group + fill_count == gr.max_group_size()
                    || is_row_finished
                {
                    (callbacks.write_row_cb)(WriteResult::ReadyAt(&back_buf[..], byte_in_row));

                    if is_row_finished {
                        (callbacks.finish_row_cb)();
                    }

                    Ok(back_buf.len())
                } else {
                    (callbacks.write_row_cb)(WriteResult::Stored(fill_count));
                    Ok(fill_count)
                }
            }
            ByteOrder::Relaxed => {
                let to_read = min(buf.len(), bpr - byte_in_row);
                (callbacks.write_row_cb)(WriteResult::ReadyAt(&buf[..to_read], byte_in_row));
                if byte_in_row + to_read == bpr {
                    (callbacks.finish_row_cb)();
                }
                Ok(to_read)
            }
        };

        self.back_buf = Some(back_buf);

        result
    }

    pub(super) fn flush<WR: FnMut(&[u8])>(&mut self, mut callback: WR) -> std::io::Result<()> {
        let back_buf = self.back_buf.take().expect(LOST_BUFFER_MESSAGE);
        (callback)(&back_buf[..self.avail]);
        self.avail = 0;
        self.back_buf = Some(back_buf);

        Ok(())
    }
}

pub(crate) enum WriteResult<'a> {
    Stored(usize),
    /// Second field is byte number in row
    ReadyAt(&'a [u8], usize),
}

impl<'a> WriteResult<'a> {
    pub(crate) fn bytes_processed(&self) -> usize {
        match self {
            WriteResult::Stored(s) => *s,
            WriteResult::ReadyAt(b, _) => b.len(),
        }
    }
}

pub(super) struct TextWriter<C: CharFormatting> {
    fmt: C,

    buf: Vec<u8>,
    avail: usize,
}

impl<C: CharFormatting> TextWriter<C> {
    pub(super) fn new(fmt: C, max_bytes: usize) -> Self {
        Self {
            fmt,
            buf: vec![0; max_bytes],
            avail: 0,
        }
    }
}

impl<C: CharFormatting> TextWriter<C> {
    pub(super) fn write<O: Write>(&mut self, bytes: &[u8], out: &mut O) -> Result<usize> {
        let len = min(bytes.len(), self.buf.len() - self.avail);

        let mut tmp = &bytes[..];
        tmp.read_exact(&mut self.buf[self.avail..self.avail + len])?;

        self.avail += len;

        if self.avail == self.buf.len() {
            let s = self.fmt.format(&self.buf);
            let _ = out.write_all(s.as_bytes())?;

            self.avail = 0;
        }

        Ok(len)
    }

    pub(super) fn flush<O: Write>(&mut self, out: &mut O) -> Result<()> {
        todo!()
    }

    pub(super) fn has_data(&self) -> bool {
        self.avail != 0
    }
}
