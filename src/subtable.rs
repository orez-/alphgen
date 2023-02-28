use byteorder::{BigEndian, WriteBytesExt};
use std::io::{self, Cursor, Write};

/// Writer adapter for font subtables.
/// That is, tables of the form:
///     num_entries: u16
///     byte_offset_to_entries: Vec<u16>
///     entries: <variable>
pub(crate) struct Subtable<W: Write> {
    writer: W,
    body_offset: u16,
    body: Cursor<Vec<u8>>,
}

impl<W: Write> Subtable<W> {
    pub fn new(writer: W, body_offset: u16) -> Subtable<W> {
        Subtable {
            writer,
            body_offset,
            body: Default::default(),
        }
    }

    pub fn header(&mut self) -> SubtableHeader<'_, W> {
        SubtableHeader(self)
    }

    pub fn body(&mut self) -> impl Write + '_ {
        &mut self.body
    }

    pub fn finalize(mut self) -> io::Result<()> {
        self.writer.write_all(&self.body.into_inner())
    }
}

pub(crate) struct SubtableHeader<'a, W: Write>(&'a mut Subtable<W>);

impl<'a, W: Write> SubtableHeader<'a, W> {
    pub fn mark_offset(&mut self) -> io::Result<()> {
        let pos = self.0.body.position() as u16;
        self.write_u16::<BigEndian>(pos + self.0.body_offset)
    }
}

impl<'a, W: Write> Write for SubtableHeader<'a, W> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.writer.write(bytes)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.writer.flush()
    }
}
