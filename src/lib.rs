mod tables;
mod push_bytes;

use crate::tables::{CMap, Gylf, Head, Name};
// use crate::push_bytes::PushBytes;
use serde::{Serialize, Serializer};

// shortFrac   16-bit signed fraction
// Fixed   16.16-bit signed fixed-point number
// FWord   16-bit signed integer that describes a quantity in FUnits, the smallest measurable distance in em space.
// uFWord  16-bit unsigned integer that describes a quantity in FUnits, the smallest measurable distance in em space.
// F2Dot14 16-bit signed fixed number with the low 14 bits representing fraction.
// longDateTime    The long internal format of a date in seconds since 12:00 midnight, January 1, 1904. It is represented as a signed 64-bit integer.

// > A table is a sequence of words. Each table must be long aligned and padded with zeroes if necessary.
// https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6.html
struct Font {
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

impl Serialize for Font {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        // Header
        // u32 - magic
        let mut buf = vec![0x00, 0x01, 0x00, 0x00];
        // u16 - number of tables
        // u16 - search range
        // u16 - entry selector
        // u16 - range shift
        serializer.serialize_bytes(&buf)?;

        // Table Records
        // xxx: don't reuse the serializer, we gotta buffer it somewhere else first.
        self.cmap.serialize(serializer)
    }
}

// impl BitmapFont {
// }

// Bit-aligned bitmap data padded to byte boundaries.

// Optional tables
// 'cvt '  control value
// 'fpgm'  font program
// 'hdmx'  horizontal device metrics
// 'kern'  kerning
// 'OS/2'  OS/2
// 'prep'  control value program

type Bitmap<'a> = &'a [u8];

pub fn bitmap_font<'a, G, L>(glyphs: G, ligatures: L) -> Font
where
    G: IntoIterator<Item=(char, Bitmap<'a>)>,
    L: IntoIterator<Item=(&'a str, Bitmap<'a>)>,
{
    todo!();
}
