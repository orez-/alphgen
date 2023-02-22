// https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6cmap.html
use crate::{FontTable, TableWriter};
use crate::bsearch::BSearch;
use std::io::{self, Write};
use std::iter::zip;
use byteorder::{BigEndian, WriteBytesExt};
use crate::platform::Platform;
use crate::itertools::split_when;

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

    pub(crate) fn from_char_order(order: &[char]) -> Result<Self, ()> {
        let order: Result<Vec<u16>, _> = order.into_iter().map(|&c| to_u16(c)).collect();
        let mut glyph_idx = 1;
        let segments: Vec<_> = split_when(&order?, |&a, &b| a + 1 != b)
            .map(|slice| {
                let start = *slice.first().expect("`split_when` should generate non-empty slices");
                let end = *slice.last().expect("`split_when` should generate non-empty slices");
                let range = (end - start + 1) as i16;
                let delta = glyph_idx - start as i16;
                glyph_idx += range;
                Segment { start, end, delta }
            }).chain([Segment::end_cap()])
            .collect();

        let record = CMapSubtableRecord {
            platform: Platform::unicode_2_0(),
            subtable: CMapSubtable::Format4 {
                language_id: 0,
                segments,
            }
        };
        Ok(CMap { subtables: vec![record] })
    }
}

// TODO: a real-ass error type
fn to_u16(c: char) -> Result<u16, ()> {
    let full = c as u32;
    full.try_into().map_err(|_| ())
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
    },
    Format4 {
        language_id: u16,
        segments: Vec<Segment>,
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
            CMapSubtable::Format4 { language_id, segments } => {
                let seg_count = segments.len() as u16;
                let bsearch = BSearch::from(seg_count, 2);
                let subtable_size = 16 + 8 * seg_count;

                buf.write_u16::<BigEndian>(0x0004)?;  // format
                buf.write_u16::<BigEndian>(subtable_size)?;
                buf.write_u16::<BigEndian>(*language_id)?;
                buf.write_u16::<BigEndian>(bsearch.len)?;
                buf.write_u16::<BigEndian>(bsearch.search_range)?;
                buf.write_u16::<BigEndian>(bsearch.entry_selector)?;
                buf.write_u16::<BigEndian>(bsearch.range_shift)?;
                for segment in segments {
                    buf.write_u16::<BigEndian>(segment.end)?;
                }
                buf.write_u16::<BigEndian>(0x0000)?;  // reserved pad
                for segment in segments {
                    buf.write_u16::<BigEndian>(segment.start)?;
                }
                for segment in segments {
                    buf.write_i16::<BigEndian>(segment.delta)?;
                }
                // idRangeOffsets, but I do not understand how this could be helpful
                // except to complicate your font parsing.
                for _ in segments {
                    buf.write_u16::<BigEndian>(0)?;
                }
                // glyph_id_array goes here but I do not understand its purpose.
                // we "hardcode" it to empty.
            }
        }
        Ok(buf.len() - original_len)
    }
}

/// A `Segment` describes a range of character codes, and maps them
/// to glyph indexes.
///
/// eg: a Segment over the character range 0x61..=0x7A ('a'..='z')
/// with delta -0x60 maps these characters to glyphs 1..=26.
struct Segment {
    start: u16,
    end: u16,
    delta: i16,
}

impl Segment {
    fn end_cap() -> Self {
        Segment {
            start: 0xffff,
            end: 0xffff,
            delta: 1,
        }
    }
}
