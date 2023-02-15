// https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6cmap.html
use crate::{FontTable, TableWriter};
use std::io::{self, Write};
use std::iter::zip;
use byteorder::{BigEndian, WriteBytesExt};
use crate::platform::Platform;

pub(crate) struct CMap {
    subtables: Vec<CMapSubtableRecord>,
}

impl CMap {
    pub(crate) fn from_ascii_order(order: &[char]) -> Result<Self, ()> {
        assert!(order.len() <= 255);

        let mut glyph_indexes = [0; 256];
        for (loca_idx, &chr) in zip(1.., order) {
            let gidx: u8 = chr.try_into()
                .map_err(|_| ())?;
            let gidx = gidx as usize;
            glyph_indexes[gidx] = loca_idx;
        }

        let record = CMapSubtableRecord {
            platform: Platform::unicode_2_0(),
            subtable: CMapSubtable::Format0 {
                language_id: 0,
                glyph_indexes,
            }
        };
        Ok(CMap { subtables: vec![record] })
    }
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
            let [platform_id, encoding_id] = record.platform.to_bytes();
            writer.write_u16::<BigEndian>(platform_id)?;
            writer.write_u16::<BigEndian>(encoding_id)?;
            writer.write_u32::<BigEndian>(offset)?;
            offset += record.subtable.write(&mut subtables)? as u32;
        }
        writer.write_all(&subtables)?;
        Ok(())
    }
}

struct CMapSubtableRecord {
    platform: Platform,
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
