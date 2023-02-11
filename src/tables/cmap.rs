// https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6cmap.html
use crate::push_bytes::PushBytes;
use serde::{Serialize, Serializer};

pub(crate) struct CMap {
    subtables: Vec<CMapSubtableRecord>,
}

impl Serialize for CMap {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut buf = Vec::new();
        let mut subtables = Vec::new();
        // version
        buf.push_be_u16(0x0000);
        let len = self.subtables.len() as u16;
        buf.push_be_u16(len);

        // 4 bytes for version + len,
        // 8 bytes for each encoding record
        let mut offset = 4 + len as u32 * 8;

        // encoding records
        for record in &self.subtables {
            buf.push_be_u16(record.platform_id);
            buf.push_be_u16(record.encoding_id);
            buf.push_be_u32(offset);
            let subtable = record.subtable.serialize();
            offset += subtable.len() as u32;
            subtables.extend(subtable);
        }
        buf.extend(subtables);
        serializer.serialize_bytes(&buf)
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

// XXX: hey, wait a second,
// this isn't related to serde at all!
impl CMapSubtable {
    fn serialize(&self) -> Vec<u8> {
        let mut out = Vec::new();
        match self {
            CMapSubtable::Format0 { language_id, glyph_indexes } => {
                out.push_be_u16(0x0000);  // format
                out.push_be_u16(0x0106);  // subtable size
                out.push_be_u16(*language_id);
                out.extend(glyph_indexes);
            }
        }
        out
    }
}
