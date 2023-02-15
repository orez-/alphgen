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

/// A writer adapter struct for font tables.
///
/// Each font table must be u32-aligned: any trailing bytes need to be
/// 0x00 padded. Additionally, each table record tracks the table's
/// length and its checksum, literally a (wrapping) u32 sum of its
/// u32 chunks.
///
/// This adapter manages these requirements: the `finalize` method
/// pads the result and returns a tuple of the checksum and length.
pub(crate) struct TableWriter<'a, W: Write> {
    writer: &'a mut W,
    checksum: u32,
    in_progress_word: [u8; 4],
    length: usize,
}

impl<'a, W: Write> TableWriter<'a, W> {
    pub fn new(writer: &'a mut W) -> TableWriter<'a, W> {
        Self {
            writer,
            checksum: 0,
            in_progress_word: [0, 0, 0, 0],
            length: 0,
        }
    }

    pub fn finalize(mut self) -> io::Result<(u32, u32)> {
        let count = (4 - self.length % 4) % 4;
        if count != 0 {
            self.write_all(&vec![0; count])?;
        }
        Ok((self.checksum, self.length as u32))
    }
}

impl<'a, W: Write> Write for TableWriter<'a, W> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let out = self.writer.write(bytes)?;
        if bytes.is_empty() { return Ok(out); }
        let bytes = &bytes[..out];

        // front
        let len = self.length % 4;
        let to_take = bytes.len().min(4 - len);
        let (front, bytes) = bytes.split_at(to_take);
        if len != 0 {
            self.in_progress_word[len..][..to_take]
                .copy_from_slice(front);
            self.length += to_take;
            if len + to_take < 4 {
                return Ok(out);
            }
            let word = u32::from_be_bytes(self.in_progress_word);
            self.checksum = self.checksum.wrapping_add(word);
        }

        // middle
        // XXX: array_chunks is nightly as of 1.67
        let mut chunks = bytes.chunks_exact(4);
        for chunk in chunks.by_ref() {
            let chunk = chunk.try_into().unwrap();
            let word = u32::from_be_bytes(chunk);
            self.checksum = self.checksum.wrapping_add(word);
        }

        // end
        let rem = chunks.remainder();
        self.in_progress_word[..rem.len()]
            .copy_from_slice(rem);

        Ok(out)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

/// Writer adapter to count the
pub(crate) struct CountWriter<W: Write> {
    writer: W,
    count: usize,
}

impl<W: Write> CountWriter<W> {
    pub fn from(writer: W) -> Self {
        CountWriter {
            writer,
            count: 0,
        }
    }

    pub fn count(&self) -> usize {
        self.count
    }

    pub fn into_inner(self) -> W {
        self.writer
    }
}

impl CountWriter<io::Sink> {
    pub fn sink() -> Self {
        Self::from(io::sink())
    }
}

impl<W: Write> Write for CountWriter<W> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let len = self.writer.write(bytes)?;
        self.count += len;
        Ok(len)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}
