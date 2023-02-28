// https://learn.microsoft.com/en-us/typography/opentype/spec/gsub

use bitflags::bitflags;
use byteorder::{BigEndian, WriteBytesExt};
use crate::{FontTable, GlyphId, TableWriter};
use crate::itertools::split_when;
use crate::subtable::SubtableBuffer;
use std::io::{self, Write};

pub(crate) struct GSub {
    scripts: ScriptListTable,
    features: FeatureListTable,
    lookup: LookupListTable,
}

impl FontTable for GSub {
    const TAG: &'static [u8; 4] = b"GSUB";

    fn write<W: Write>(&self, writer: &mut TableWriter<W>) -> io::Result<()> {
        let mut subtable = SubtableBuffer::new(14);
        {
            let mut header = subtable.header();
            header.write_u32::<BigEndian>(0x00010000)?;  // version
            header.mark_offset()?;  // scripts offset
        }
        self.scripts.write(subtable.body())?;
        subtable.header().mark_offset()?;  // features offset
        self.features.write(subtable.body())?;
        subtable.header().mark_offset()?;  // lookup offset
        self.lookup.write(subtable.body())?;
        subtable.write(writer)
    }
}

impl GSub {
    pub fn new() -> Self {
        GSub {
            scripts: ScriptListTable,
            features: FeatureListTable,
            lookup: LookupListTable { list: Vec::new() },
        }
    }
}

struct LookupListTable {
    list: Vec<LookupTable>
}

impl LookupListTable {
    // https://learn.microsoft.com/en-us/typography/opentype/spec/chapter2#lookup-list-table
    fn write<W: Write>(&self, writer: W) -> io::Result<()> {
        // intentionally 0: it's offset from the start of the list itself.
        let mut subtable = SubtableBuffer::new(0);
        subtable.header().write_u16::<BigEndian>(self.list.len() as u16)?;
        for tbl in &self.list {
            subtable.header().mark_offset()?;
            tbl.write(&mut subtable.body())?;
        }
        subtable.write(writer)
    }
}

struct LookupTable {
    lookup_flag: LookupFlags,
    subtable: LookupSubtable,
}

impl LookupTable {
    // https://learn.microsoft.com/en-us/typography/opentype/spec/chapter2#lookup-table
    fn write<W: Write>(&self, writer: W) -> io::Result<()> {
        // As near as i can tell, there's no meaningful distinction between
        // the list of subtables here and the list of subtables on the next layer down.
        // It's not like this one can be a mix of different types or anything,
        // the type is homogeneous on _this_ level! 🙃
        //
        // Hardcode table len 1 here i guess!!
        let subtable_count = 1;

        // XXX: +2 if markFilteringSet. see below
        let offset = 6 + 2 * subtable_count;
        let mut subtable = SubtableBuffer::new(offset);
        let lookup_type = self.subtable.lookup_type();
        {
            let mut header = subtable.header();
            header.write_u16::<BigEndian>(lookup_type)?;
            header.write_u16::<BigEndian>(self.lookup_flag.bits)?;
            header.write_u16::<BigEndian>(subtable_count)?;
            header.mark_offset()?;
        }
        self.subtable.write(subtable.body())?;

        // XXX: we'd need to write a markFilteringSet u16 here when
        // the corresponding LookupFlag is set. Currently unsupported.
        subtable.write(writer)
    }
}

bitflags! {
    struct LookupFlags: u16 {
        const RIGHT_TO_LEFT = 1 << 0;
        const IGNORE_BASE_GLYPHS = 1 << 1;
        const IGNORE_LIGATURES = 1 << 2;
        const IGNORE_MARKS = 1 << 3;
        // const USE_MARK_FILTERING_SET = 1 << 4;
        // const MARK_ATTACHMENT_TYPE_MASK = 0xFF00;  // ????????
    }
}

enum LookupSubtable {
    // TODO: these gotta be sorted.
    // how do we enforce this?
    LigatureSubst(Vec<Ligature>),
}

const SET_RECORD_SIZE: u16 = 2;
type Ligature = (Vec<GlyphId>, GlyphId);

impl LookupSubtable {
    fn lookup_type(&self) -> u16 {
        match self {
            Self::LigatureSubst(_) => 4,
        }
    }

    fn write<W: Write>(&self, writer: W) -> io::Result<()> {
        match self {
            // https://learn.microsoft.com/en-us/typography/opentype/spec/gsub#41-ligature-substitution-format-1
            Self::LigatureSubst(subs) => {
                let ligature_sets: Vec<&[Ligature]> = split_when(
                    subs,
                    |(a, _), (b, _)| a.first() != b.first()
                ).collect();
                let ligature_set_count = ligature_sets.len() as u16;
                // offset to coverage table
                let offset = 4 + SET_RECORD_SIZE * ligature_set_count;
                let mut subtable = SubtableBuffer::new(offset);
                {
                    let mut header = subtable.header();
                    header.write_u16::<BigEndian>(1)?;  // version
                    header.mark_offset()?;  // offset to coverage table
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
                    write_ligature_set(set, subtable.body())?;
                }
                subtable.write(writer)?;
            }
        }
        Ok(())
    }
}

fn write_ligature_set<W: Write>(set: &[Ligature], writer: W) -> io::Result<()> {
    let lig_count = set.len() as u16;
    let offset = 2 + lig_count * 2;
    let mut ligset = SubtableBuffer::new(offset);
    ligset.header().write_u16::<BigEndian>(lig_count)?;
    for &(ref pattern, replacement) in set {
        ligset.header().mark_offset()?;  // ligature offset
        let mut body = ligset.body();
        body.write_u16::<BigEndian>(replacement.0)?;  // output glyph id
        body.write_u16::<BigEndian>(pattern.len() as u16)?;  // component count
        // note that we skip the first glyph in the ligature,
        // since it's encoded in the Coverage table
        for &glyph in &pattern[1..] {
            body.write_u16::<BigEndian>(glyph.0)?;  // component glyph id
        }
    }
    ligset.write(writer)
}

// ===

enum Coverage {
    List(Vec<GlyphId>),
    // Ranges
}

impl Coverage {
    fn write<W: Write>(&self, mut writer: W) -> io::Result<()> {
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

// ===

struct ScriptListTable;
impl ScriptListTable {
    // https://learn.microsoft.com/en-us/typography/opentype/spec/chapter2#slTbl_sRec
    fn write<W: Write>(&self, mut writer: W) -> io::Result<()> {
        writer.write_u16::<BigEndian>(0)  // number of records
    }
}

struct FeatureListTable;
impl FeatureListTable {
    // https://learn.microsoft.com/en-us/typography/opentype/spec/chapter2#flTbl
    fn write<W: Write>(&self, mut writer: W) -> io::Result<()> {
        writer.write_u16::<BigEndian>(0)  // number of records
    }
}
