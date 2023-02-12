use std::io::{self, Seek, SeekFrom, Write};
use SeekFrom::Start;

// XXX: i don't like this API.
// - `split_at` implies separation between
//   two sections of a buffer, but you can absolutely bleed over into
//   the other if you're not careful.
// - `swap` feels stateful [derogatory] and footgun-ish
// Also more generally I don't know if using Seek this way is copacetic.

/// A two-headed Write-r.
/// Use `writer.swap()` to swap between the two heads.
pub(crate) struct TwoWrite<'a, W: Write + Seek> {
    writer: &'a mut W,
    other_head: u64,
}

impl<'a, W: Write + Seek> TwoWrite<'a, W> {
    pub fn split_at(writer: &'a mut W, idx: u64) -> Self {
        Self {
            writer,
            other_head: idx,
        }
    }

    pub fn swap(&mut self) -> io::Result<()> {
        let pos = self.writer.stream_position()?;
        self.writer.seek(Start(self.other_head))?;
        self.other_head = pos;
        Ok(())
    }
}

impl<'a, W: Write + Seek> Write for TwoWrite<'a, W> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.writer.write(bytes)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

impl<'a, W: Write + Seek> Seek for TwoWrite<'a, W> {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, std::io::Error> {
        self.writer.seek(pos)
    }
}
