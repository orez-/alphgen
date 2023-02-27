// https://learn.microsoft.com/en-us/typography/opentype/spec/gsub

use bitflags::bitflags;
use byteorder::{BigEndian, WriteBytesExt};
use crate::{FontTable, GlyphId, TableWriter};
use crate::itertools::split_when;
use crate::subtable::Subtable;
use std::io::{self, Write, Cursor};

pub(crate) struct GSub {
    // scripts:
    // features:
    // lookup:
}

impl FontTable for GSub {
    const TAG: &'static [u8; 4] = b"GSUB";

    fn write<W: Write>(&self, writer: &mut TableWriter<W>) -> io::Result<()> {
        writer.write_u32::<BigEndian>(0x00010000)?;  // version
        // scripts:  // offset u16 from gsub start
        // features: // offset u16 from gsub start
        // lookup: // offset u16 from gsub start
        Ok(())
    }
}

struct LookupTableList {
    list: Vec<LookupTable>
}

struct LookupTable {
    // lookup_type: u16,  // Different enumerations for GSUB and GPOS
    lookup_flag: LookupFlags,  // Lookup qualifiers
    // sub_table_count: u16,  // Number of subtables for this lookup
    // subtable_offsets: u16,  // Array of offsets to lookup subtables, from beginning of Lookup table
    subtable: LookupSubtable,
    mark_filtering_set: u16,
}

impl LookupTable {
    fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        let lookup_type = self.subtable.lookup_type();
        writer.write_u16::<BigEndian>(lookup_type)?;
        Ok(())
    }
}

bitflags! {
    struct LookupFlags: u16 {
        const RIGHT_TO_LEFT = 1 << 0;
        const IGNORE_BASE_GLYPHS = 1 << 1;
        const IGNORE_LIGATURES = 1 << 2;
        const IGNORE_MARKS = 1 << 3;
        const USE_MARK_FILTERING_SET = 1 << 4;
        // const MARK_ATTACHMENT_TYPE_MASK = 0xFF00;  // ????????
    }
}

enum LookupSubtable {
    // TODO: these gotta be sorted.
    // how do we enforce this?
    LigatureSubst(Vec<(Vec<GlyphId>, GlyphId)>),
}

const SET_RECORD_SIZE: u16 = 2;

impl LookupSubtable {
    fn lookup_type(&self) -> u16 {
        match self {
            Self::LigatureSubst(_) => 4,
        }
    }

    fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        match self {
            Self::LigatureSubst(subs) => {
                let ligature_sets: Vec<_> = split_when(
                    subs,
                    |(a, _), (b, _)| a.first() != b.first()
                ).collect();
                let ligature_set_count = ligature_sets.len() as u16;
                // offset to coverage table
                let offset = 4 + SET_RECORD_SIZE * ligature_set_count;
                let mut subtable = Subtable::new(writer, offset);
                {
                    let mut header = subtable.header();
                    header.write_u16::<BigEndian>(1)?;  // version
                    header.mark_offset()?;
                    header.write_u16::<BigEndian>(ligature_set_count)?;
                }

                let coverage = Coverage::List(
                    ligature_sets
                        .iter()
                        .map(|set| *set.first()
                            .and_then(|(pattern, _)| pattern.first())
                            .expect("ligature pattern should not be empty"))
                        .collect()
                );
                coverage.write(&mut subtable.body())?;

                for &set in &ligature_sets {
                    subtable.header().mark_offset()?;
                    let lig_count = set.len() as u16;
                    let offset = 2 + lig_count * 2;
                    let mut ligset = Subtable::new(subtable.body(), offset);
                    ligset.header().write_u16::<BigEndian>(lig_count)?;
                    for &(ref pattern, replacement) in set {
                        ligset.header().mark_offset()?;
                        let mut body = ligset.body();
                        body.write_u16::<BigEndian>(replacement.0)?;
                        body.write_u16::<BigEndian>(pattern.len() as u16)?;
                        for &glyph in &pattern[1..] {
                            body.write_u16::<BigEndian>(glyph.0)?;
                        }
                    }
                    ligset.finalize()?;
                }
                subtable.finalize()?;
            }
        }
        Ok(())
    }
}

// ===

enum Coverage {
    List(Vec<GlyphId>),
    // Ranges
}

impl Coverage {
    fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        match self {
            // https://learn.microsoft.com/en-us/typography/opentype/spec/chapter2#coverage-format-1
            Coverage::List(glyph_ids) => {
                writer.write_u16::<BigEndian>(1)?;  // coverage format
                writer.write_u16::<BigEndian>(glyph_ids.len() as u16)?;
                for &GlyphId(id) in glyph_ids {
                    writer.write_u16::<BigEndian>(id)?;
                }
            }
        }
        Ok(())
    }
}
