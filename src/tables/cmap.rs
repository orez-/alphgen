// https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6cmap.html
use crate::{FontTable, TableWriter};
use std::io::{self, Write};
use byteorder::{BigEndian, WriteBytesExt};

pub(crate) struct CMap {
    subtables: Vec<CMapSubtableRecord>,
}

impl FontTable for CMap {
    const TAG: &'static [u8; 4] = b"cmap";

    fn write<W: Write>(&self, writer: &mut TableWriter<W>) -> io::Result<()> {

        let mut subtables = Vec::new();
        // version
        writer.write_u16::<BigEndian>(0x0000)?;
        let len = self.subtables.len() as u16;
        writer.write_u16::<BigEndian>(len)?;

        // 4 bytes for version + len,
        // 8 bytes for each encoding record
        let mut offset = 4 + len as u32 * 8;

        // encoding records
        for record in &self.subtables {
            writer.write_u16::<BigEndian>(record.platform_id)?;
            writer.write_u16::<BigEndian>(record.encoding_id)?;
            writer.write_u32::<BigEndian>(offset)?;
            offset += record.subtable.write(&mut subtables)? as u32;
        }
        writer.write_all(&subtables)?;
        Ok(())
    }
}

struct CMapSubtableRecord {
    platform_id: u16,
    encoding_id: u16,
    subtable: CMapSubtable,
}

enum CMapSubtable {
    Format0 {
        language_id: u16,
        glyph_indexes: [u8; 256],
    }
}

impl CMapSubtable {
    fn write(&self, buf: &mut Vec<u8>) -> io::Result<usize> {
        let original_len = buf.len();
        match self {
            CMapSubtable::Format0 { language_id, glyph_indexes } => {
                buf.write_u16::<BigEndian>(0x0000)?;  // format
                buf.write_u16::<BigEndian>(0x0106)?;  // subtable size
                buf.write_u16::<BigEndian>(*language_id)?;
                buf.extend(glyph_indexes);
            }
        }
        Ok(buf.len() - original_len)
    }
}
