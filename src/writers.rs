use super::format::*;
use std::cmp::min;
use std::io::*;

const LOST_BUFFER_MESSAGE: &str = "Somewhere we lost back buffer";

pub(super) struct Callbacks<
    SR: FnMut() -> Result<()>,
    WR: FnMut(WriteResult) -> Result<()>,
    FR: FnMut() -> Result<()>,
> {
    start_row_cb: SR,
    write_row_cb: WR,
    finish_row_cb: FR,
}

impl<SR, WR, FR> Callbacks<SR, WR, FR>
where
    SR: FnMut() -> Result<()>,
    WR: FnMut(WriteResult) -> Result<()>,
    FR: FnMut() -> Result<()>,
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
        SR: FnMut() -> Result<()>,
        WR: FnMut(WriteResult) -> Result<()>,
        FR: FnMut() -> Result<()>,
    {
        let mut back_buf = self.back_buf.take().expect(LOST_BUFFER_MESSAGE);

        let gr = &self.groupping;
        let bpr = gr.bytes_per_row();
        let byte_in_row = self.address % bpr;

        let result = {
            if byte_in_row == 0 {
                (callbacks.start_row_cb)()?;
            }

            match self.order {
                ByteOrder::Strict => {
                    assert!(
                        back_buf.len() > 0,
                        "Back buffer is zero length while ByteOrder::Strict"
                    );

                    let remaining = gr.bytes_left_in_group_after(byte_in_row);
                    let byte_in_group = gr.byte_number_in_group(byte_in_row);
                    let fill_count = min(remaining, buf.len());
                    self.avail += fill_count;

                    let mut buf_to_read = &mut back_buf[byte_in_group..byte_in_group + fill_count];
                    let mut buf = &buf[..];
                    buf.read_exact(&mut buf_to_read)
                        .expect("Could not read from buffer");

                    let is_row_finished = byte_in_row + fill_count == bpr;

                    if self.avail == gr.max_group_size() || is_row_finished {
                        (callbacks.write_row_cb)(WriteResult::ReadyAt(&buf_to_read[..], byte_in_row))?;

                        if is_row_finished {
                            (callbacks.finish_row_cb)()?;
                        }

                        self.avail = 0;

                        self.address += back_buf.len();
                        Ok(back_buf.len())
                    } else {
                        (callbacks.write_row_cb)(WriteResult::Stored(fill_count))?;

                        self.address += fill_count;
                        Ok(fill_count)
                    }
                }
                ByteOrder::Relaxed => {
                    let to_read = min(buf.len(), bpr - byte_in_row);
                    (callbacks.write_row_cb)(WriteResult::ReadyAt(&buf[..to_read], byte_in_row))?;
                    if byte_in_row + to_read == bpr {
                        (callbacks.finish_row_cb)()?;
                    }

                    self.address += to_read;
                    Ok(to_read)
                }
            }
        };
        self.back_buf = Some(back_buf);

        result
    }

    /// Callback takes buffer and byte number past last byte
    pub(super) fn flush<WR: FnMut(&[u8], usize) -> Result<()>> (
        &mut self,
        mut callback: WR,
    ) -> std::io::Result<()> {
        let back_buf = self.back_buf.take().expect(LOST_BUFFER_MESSAGE);

        let byte_in_row = self.address % self.groupping.bytes_per_row();
        let result = (callback)(&back_buf[..self.avail], byte_in_row);
        self.avail = 0;

        self.back_buf = Some(back_buf);

        result
    }
}

pub(crate) enum WriteResult<'a> {
    Stored(usize),
    /// Second field is byte number in row
    ReadyAt(&'a [u8], usize),
}

pub(super) struct TextWriter<C: CharFormatting> {
    fmt: C,

    result: String,
    avail: usize,
    max_bytes: usize,
}

impl<C: CharFormatting> TextWriter<C> {
    pub(super) fn new(fmt: C, max_bytes: usize) -> Self {
        Self {
            fmt,
            result: String::new(),
            avail: 0,
            max_bytes,
        }
    }
}

impl<C: CharFormatting> TextWriter<C> {
    pub(super) fn write(&mut self, bytes: &[u8]) -> Result<usize> {
        assert!(
            self.avail + bytes.len() <= self.max_bytes,
            "Text writer received too much bytes before starting new row"
        );

        self.avail += bytes.len();

        let s = self.fmt.format(bytes);
        self.result += &s;

        Ok(bytes.len())
    }

    pub(super) fn take_result(&mut self) -> String {
        let mut result = (&self.result).to_string();

        let tail = self.fmt.padding_string(self.max_bytes - self.avail);
        result += &tail;

        self.result = String::new();
        self.avail = 0;

        result
    }

    pub(super) fn has_data(&self) -> bool {
        self.avail > 0
    }
}
