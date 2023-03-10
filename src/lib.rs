mod bsearch;
mod itertools;
mod platform;
mod sprite;
mod subtable;
mod tables;
mod time;
mod writeutils;

use std::path::Path;
use std::fs::File;
use std::io::{self, Seek, Write};
use byteorder::{BigEndian, WriteBytesExt};
use bsearch::BSearch;
use crate::sprite::Sprite;
use crate::tables::{CMap, Glyf, GSub, Head, HHea, HMtx, Loca, MaxP, Name, Os2, Post, name};
use crate::writeutils::{TableWriter, TwoWrite};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct GlyphId(u16);

const RECORD_SIZE: u16 = 16;

// shortFrac   16-bit signed fraction
// Fixed   16.16-bit signed fixed-point number
// FWord   16-bit signed integer that describes a quantity in FUnits, the smallest measurable distance in em space.
// uFWord  16-bit unsigned integer that describes a quantity in FUnits, the smallest measurable distance in em space.
// F2Dot14 16-bit signed fixed number with the low 14 bits representing fraction.
// longDateTime    The long internal format of a date in seconds since 12:00 midnight, January 1, 1904. It is represented as a signed 64-bit integer.

// > A table is a sequence of words. Each table must be long aligned and padded with zeroes if necessary.
// https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6.html
pub struct Font {
    cmap: CMap, // character to glyph mapping
    glyf: Glyf, // glyph data
    gsub: Option<GSub>, // ligature data
    head: Head, // font header
    hhea: HHea, // horizontal header
    hmtx: HMtx, // horizontal metrics
    loca: Loca, // index to location
    maxp: MaxP, // maximum profile
    name: Name, // naming
    os2: Os2, // windows junk
    post: Post, // PostScript
}

impl Font {
    /// Save the font to a file at the path specified.
    pub fn save<Q: AsRef<Path>>(&self, path: Q) -> io::Result<()> {
        let mut file = File::create(path)?;
        self.write_to(&mut file)?;
        Ok(())
    }

    /// Encode this font and write it to `writer`.
    pub fn write_to<W: Write + Seek>(&self, writer: &mut W) -> io::Result<()> {
        // TODO: I don't know what the best way to represent these tables is,
        // but hardcoding the length like this is almost surely Not It,
        // especially now that it's starting to be conditional.
        let table_count = 10 + self.gsub.is_some() as u16;
        let bsearch = BSearch::from(table_count, RECORD_SIZE);
        let table_ptr = 12 + (table_count * RECORD_SIZE) as u64;
        let mut writer = TwoWrite::split_at(writer, table_ptr);

        // Header
        writer.write_all(&[0x00, 0x01, 0x00, 0x00])?;  // magic
        writer.write_u16::<BigEndian>(table_count)?;
        writer.write_u16::<BigEndian>(bsearch.search_range)?;
        writer.write_u16::<BigEndian>(bsearch.entry_selector)?;
        writer.write_u16::<BigEndian>(bsearch.range_shift)?;

        writer.swap()?;
        // Table Records
        if let Some(gsub) = &self.gsub {
            self.write_table(&mut writer, gsub)?;
        }
        self.write_table(&mut writer, &self.os2)?;
        self.write_table(&mut writer, &self.cmap)?;
        self.write_table(&mut writer, &self.glyf)?;
        self.write_table(&mut writer, &self.head)?;
        self.write_table(&mut writer, &self.hhea)?;
        self.write_table(&mut writer, &self.hmtx)?;
        self.write_table(&mut writer, &self.loca)?;
        self.write_table(&mut writer, &self.maxp)?;
        self.write_table(&mut writer, &self.name)?;
        self.write_table(&mut writer, &self.post)?;
        Ok(())
    }

    fn write_table<W: Write + Seek, T: FontTable>(&self, writer: &mut TwoWrite<W>, table: &T) -> io::Result<()> {
        let table_ptr = writer.stream_position()? as u32;
        let mut twriter = TableWriter::new(writer);
        table.write(&mut twriter)?;
        let (checksum, length) = twriter.finalize()?;
        writer.swap()?;

        writer.write_all(T::TAG)?;
        writer.write_u32::<BigEndian>(checksum)?;
        writer.write_u32::<BigEndian>(table_ptr)?;
        writer.write_u32::<BigEndian>(length)?;
        writer.swap()?;
        Ok(())
    }
}

trait FontTable {
    const TAG: &'static [u8; 4];
    fn write<W: Write>(&self, writer: &mut TableWriter<W>) -> io::Result<()>;
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct Rect {
    pub x_min: i16,
    pub y_min: i16,
    pub x_max: i16,
    pub y_max: i16,
}

impl Rect {
    fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_i16::<BigEndian>(self.x_min)?;
        writer.write_i16::<BigEndian>(self.y_min)?;
        writer.write_i16::<BigEndian>(self.x_max)?;
        writer.write_i16::<BigEndian>(self.y_max)?;
        Ok(())
    }
}

// Bit-aligned bitmap data padded to byte boundaries.

// Optional tables
// 'cvt '  control value
// 'fpgm'  font program
// 'hdmx'  horizontal device metrics
// 'kern'  kerning
// 'OS/2'  OS/2
// 'prep'  control value program

type Bitmap<'a> = &'a [u8];

pub fn bitmap_font<'a, G, L>(width: usize, height: usize, missing_glyph: Bitmap<'a>, glyphs: G, ligatures: L) -> Result<Font, ()>
where
    G: IntoIterator<Item=(char, Bitmap<'a>)>,
    L: IntoIterator<Item=(&'a str, Bitmap<'a>)>,
{
    let mut glyphs: Vec<_> = glyphs.into_iter().collect();
    glyphs.sort();
    let (chars, glyph_bitmaps): (Vec<_>, Vec<_>) = glyphs.into_iter().unzip();
    let (combos, ligature_bitmaps): (Vec<_>, Vec<_>) = ligatures.into_iter().unzip();
    let ligature_offset = glyph_bitmaps.len() as u16 + 1;
    let sprites = [missing_glyph].into_iter()
        .chain(glyph_bitmaps)
        .chain(ligature_bitmaps)
        .map(|bitmap| Sprite { width, height, data: bitmap.into() });
    let glyf = Glyf::from(sprites);
    let loca = glyf.generate_loca();
    let maxp = glyf.generate_maxp();
    let cmap = CMap::from_char_order(&chars)?;
    let mut head = Head::new();
    head.index_to_loc_format = loca.needs_long() as i16;  // XXX: ????
    let mut name = Name::new();
    // name.push(name::FONT_FAMILY, "My Neat Font");
    // TODO: I think some or all of these are required,
    // but this makes me sad. Revisit.
    for id in (0..7).chain([10]) {
        if id == 2 {
            name.push(2, "Regular");
        } else {
            name.push(id, "My Neat Font");
        }
    }
    let mut hhea = HHea::new();
    let hmtx = HMtx::monospace(width as u16, vec![0; glyf.count_glyphs()]);
    hhea.num_of_long_hor_metrics = hmtx.num_of_long_hor_metrics() as u16;

    let post = Post::from_ascii_order(&chars, glyf.count_glyphs());

    type Ligature = (Vec<GlyphId>, GlyphId);
    let ligatures: Vec<Ligature> = combos.into_iter()
        .zip(ligature_offset..)
        .map(|(seq, idx)| {
            let pattern: Vec<_> = seq.chars()
                .map(|chr|
                    chars.iter()
                        .position(|&c| c == chr)  // XXX: probably make a lookup.
                        .expect("ligature must be composed of glyphs that exist in the font")
                        as u16 + 1
                ).map(GlyphId)
                .collect();
            (pattern, GlyphId(idx))
        }).collect();
    let gsub = GSub::new(ligatures);

    let font = Font {
        cmap,
        glyf,
        gsub,
        head,
        hhea,
        hmtx,
        loca,
        maxp,
        name,
        os2: Os2::default(),
        post,
    };
    Ok(font)
}
