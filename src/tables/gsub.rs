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
        let mut subtable = SubtableBuffer::new(10);
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
    pub fn new(mut ligatures: Vec<Ligature>) -> Option<Self> {
        if ligatures.is_empty() { return None; }
        ligatures.sort_unstable();
        let list = vec![LookupTable {
            lookup_flag: LookupFlags::empty(),
            subtable: LookupSubtable::LigatureSubst(ligatures),
        }];
        let features = vec![FeatureTable {
            tag: *b"liga",
            lookup_list_indices: vec![0],
        }];
        let scripts = vec![ScriptTable {
            default: Some(LangSysTable {
                reqd_feature_idx: 0,
                feature_list_indices: Vec::new(),
            }),
            script_tag: *b"DFLT",
            lang_sys: Vec::new(),
        }];
        Some(GSub {
            scripts: ScriptListTable { scripts },
            features: FeatureListTable { features },
            lookup: LookupListTable { list },
        })
    }
}

struct LookupListTable {
    list: Vec<LookupTable>
}

impl LookupListTable {
    // https://learn.microsoft.com/en-us/typography/opentype/spec/chapter2#lookup-list-table
    fn write<W: Write>(&self, writer: W) -> io::Result<()> {
        let len = self.list.len() as u16;
        let mut subtable = SubtableBuffer::new(2 + len * 2);
        subtable.header().write_u16::<BigEndian>(len)?;
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
    // > Ligatures with more components must be stored ahead of those
    // > with fewer components in order to be found.
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
                let offset = 6 + SET_RECORD_SIZE * ligature_set_count;
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

struct ScriptListTable {
    scripts: Vec<ScriptTable>,
}

impl ScriptListTable {
    // https://learn.microsoft.com/en-us/typography/opentype/spec/chapter2#slTbl_sRec
    fn write<W: Write>(&self, writer: W) -> io::Result<()> {
        let len = self.scripts.len() as u16;
        let mut subtable = SubtableBuffer::new(2 + 6 * len);
        subtable.header().write_u16::<BigEndian>(len)?;
        for feature in &self.scripts {
            subtable.header().write_all(&feature.script_tag)?;
            subtable.header().mark_offset()?;
            feature.write(subtable.body())?;
        }
        subtable.write(writer)
    }
}

struct ScriptTable {
    script_tag: [u8; 4],
    default: Option<LangSysTable>,
    lang_sys: Vec<([u8; 4], LangSysTable)>,
}

impl ScriptTable {
    // https://learn.microsoft.com/en-us/typography/opentype/spec/chapter2#script-table-and-language-system-record
    fn write<W: Write>(&self, writer: W) -> io::Result<()> {
        let len = self.lang_sys.len() as u16;
        let mut subtable = SubtableBuffer::new(4 + 6 * len);
        if let Some(lang_sys) = &self.default {
            subtable.header().mark_offset()?;
            lang_sys.write(subtable.body())?;
        } else {
            subtable.header().write_u16::<BigEndian>(0)?;  // null
        }
        subtable.header().write_u16::<BigEndian>(len)?;
        for (tag, ls) in &self.lang_sys {
            subtable.header().write_all(tag)?;
            subtable.header().mark_offset()?;
            ls.write(subtable.body())?;
        }
        subtable.write(writer)
    }
}

struct LangSysTable {
    reqd_feature_idx: u16,
    feature_list_indices: Vec<u16>,
}

impl LangSysTable {
    // https://learn.microsoft.com/en-us/typography/opentype/spec/chapter2#language-system-table
    fn write<W: Write>(&self, mut writer: W) -> io::Result<()> {
        writer.write_u16::<BigEndian>(0)?;  // reserved null byte
        writer.write_u16::<BigEndian>(self.reqd_feature_idx)?;
        writer.write_u16::<BigEndian>(self.feature_list_indices.len() as u16)?;
        for &idx in &self.feature_list_indices {
            writer.write_u16::<BigEndian>(idx)?;
        }
        Ok(())
    }
}

struct FeatureListTable {
    features: Vec<FeatureTable>,
}

impl FeatureListTable {
    // https://learn.microsoft.com/en-us/typography/opentype/spec/chapter2#flTbl
    fn write<W: Write>(&self, writer: W) -> io::Result<()> {
        let len = self.features.len() as u16;
        let mut subtable = SubtableBuffer::new(2 + 6 * len);
        subtable.header().write_u16::<BigEndian>(len)?;
        for feature in &self.features {
            subtable.header().write_all(&feature.tag)?;
            subtable.header().mark_offset()?;
            feature.write(subtable.body())?;
        }
        subtable.write(writer)
    }
}

struct FeatureTable {
    tag: [u8; 4],  // 4-byte feature identification tag
    lookup_list_indices: Vec<u16>, // Array of indices into the LookupList — zero-based (first lookup is LookupListIndex = 0)
}

impl FeatureTable {
    fn write<W: Write>(&self, mut writer: W) -> io::Result<()> {
        // > Offset from start of Feature table to FeatureParams table,
        // > if defined for the feature and present, else NULL.
        // The FeatureParams table doesn't appear to be documented anywhere?????
        writer.write_u16::<BigEndian>(0)?;
        let len = self.lookup_list_indices.len() as u16;
        writer.write_u16::<BigEndian>(len)?;
        for &idx in &self.lookup_list_indices {
            writer.write_u16::<BigEndian>(idx)?;
        }
        Ok(())
    }
}
