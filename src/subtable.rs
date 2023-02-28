use byteorder::{BigEndian, WriteBytesExt};
use std::io::{self, Cursor, Write};

/// Utility buffer for font subtables.
/// That is, tables of the form:
///     num_entries: u16
///     byte_offset_to_entries: Vec<u16>
///     entries: <variable>
pub(crate) struct SubtableBuffer {
    body_offset: u16,
    header: Cursor<Vec<u8>>,
    body: Cursor<Vec<u8>>,
}

impl SubtableBuffer {
    pub fn new(body_offset: u16) -> SubtableBuffer {
        SubtableBuffer {
            body_offset,
            header: Default::default(),
            body: Default::default(),
        }
    }

    pub fn header(&mut self) -> SubtableHeader<'_> {
        SubtableHeader(self)
    }

    pub fn body(&mut self) -> impl Write + '_ {
        &mut self.body
    }

    pub fn write<W: Write>(self, mut writer: W) -> io::Result<()> {
        writer.write_all(&self.header.into_inner())?;
        writer.write_all(&self.body.into_inner())?;
        Ok(())
    }
}

pub(crate) struct SubtableHeader<'a>(&'a mut SubtableBuffer);

impl<'a> SubtableHeader<'a> {
    pub fn mark_offset(&mut self) -> io::Result<()> {
        let pos = self.0.body.position() as u16;
        self.write_u16::<BigEndian>(pos + self.0.body_offset)
    }
}

impl<'a> Write for SubtableHeader<'a> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.header.write(bytes)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.header.flush()
    }
}
