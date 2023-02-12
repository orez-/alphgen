mod tables;
mod writeutils;

use std::io::{self, Seek, SeekFrom, Write};
use byteorder::{BigEndian, WriteBytesExt};
use crate::tables::{CMap, Glyf, Head, Name};
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
    // loca: ?, // index to location
    // maxp: ?, // maximum profile
    name: Name, // naming
    // post: ?, // PostScript
}

impl Font {
    pub fn write_to<W: Write + Seek>(&self, writer: &mut W) -> io::Result<()> {
        // TODO: I don't know what the best way to represent these tables is,
        // but hardcoding the length like this is almost surely Not It.
        let table_count = 4;
        let table_ptr = 12 + table_count as u64 * 16;
        let mut writer = TwoWrite::split_at(writer, table_ptr);

        // Header
        // u32 - magic
        writer.write_all(&[0x00, 0x01, 0x00, 0x00])?;
        // u16 - number of tables
        writer.write_u16::<BigEndian>(table_count)?;
        // u16 - search range
        // u16 - entry selector
        // u16 - range shift

        writer.swap()?;
        // Table Records
        self.write_table(&mut writer, &self.cmap)?;
        self.write_table(&mut writer, &self.glyf)?;
        self.write_table(&mut writer, &self.head)?;
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

// Bit-aligned bitmap data padded to byte boundaries.

// Optional tables
// 'cvt '  control value
// 'fpgm'  font program
// 'hdmx'  horizontal device metrics
// 'kern'  kerning
// 'OS/2'  OS/2
// 'prep'  control value program

type Bitmap<'a> = &'a [u8];

pub fn bitmap_font<'a, G, L>(width: usize, height: usize, glyphs: G, ligatures: L) -> Font
where
    G: IntoIterator<Item=(char, Bitmap<'a>)>,
    L: IntoIterator<Item=(&'a str, Bitmap<'a>)>,
{
    todo!();
}
