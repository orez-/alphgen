mod itertools;
mod platform;
mod sprite;
mod tables;
mod writeutils;

use std::path::Path;
use std::fs::File;
use std::io::{self, Seek, Write};
use byteorder::{BigEndian, WriteBytesExt};
use crate::sprite::Sprite;
use crate::tables::{CMap, Glyf, Head, Loca, MaxP, Name, name};
use crate::writeutils::{TableWriter, TwoWrite};

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
    head: Head, // font header
    // hhea: ?, // horizontal header
    // hmtx: ?, // horizontal metrics
    loca: Loca, // index to location
    maxp: MaxP, // maximum profile
    name: Name, // naming
    // post: ?, // PostScript
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
        // but hardcoding the length like this is almost surely Not It.
        let table_count = 6;
        let table_ptr = 12 + table_count as u64 * 16;
        let mut writer = TwoWrite::split_at(writer, table_ptr);

        // Header
        // u32 - magic
        writer.write_all(&[0x00, 0x01, 0x00, 0x00])?;
        // u16 - number of tables
        writer.write_u16::<BigEndian>(table_count)?;
        // u16 - search range
        writer.write_u16::<BigEndian>(0)?;
        // u16 - entry selector
        writer.write_u16::<BigEndian>(0)?;
        // u16 - range shift
        writer.write_u16::<BigEndian>(0)?;

        writer.swap()?;
        // Table Records
        self.write_table(&mut writer, &self.cmap)?;
        self.write_table(&mut writer, &self.glyf)?;
        self.write_table(&mut writer, &self.head)?;
        self.write_table(&mut writer, &self.loca)?;
        self.write_table(&mut writer, &self.maxp)?;
        self.write_table(&mut writer, &self.name)?;
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

pub fn bitmap_font<'a, G, L>(width: usize, height: usize, missing_glyph: Bitmap<'a>, glyphs: G, _ligatures: L) -> Result<Font, ()>
where
    G: IntoIterator<Item=(char, Bitmap<'a>)>,
    L: IntoIterator<Item=(&'a str, Bitmap<'a>)>,
{
    let mut glyphs: Vec<_> = glyphs.into_iter().collect();
    glyphs.sort();
    let (chars, bitmaps): (Vec<_>, Vec<_>) = glyphs.into_iter().unzip();
    let sprites = bitmaps.into_iter()
        .chain([missing_glyph])
        .map(|bitmap| Sprite { width, height, data: bitmap.into() });
    let glyf = Glyf::from(sprites);
    let loca = glyf.generate_loca();
    let maxp = glyf.generate_maxp();
    let cmap = CMap::from_char_order(&chars)?;
    let mut head = Head::new();
    head.index_to_loc_format = loca.needs_long() as i16;  // XXX: 😬
    let mut name = Name::new();
    name.push(name::FONT_FAMILY, "My Neat Font");

    let font = Font {
        cmap,
        glyf,
        head,
        loca,
        maxp,
        name,
    };
    Ok(font)
}
