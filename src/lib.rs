// 11A80-11E7F Font Graphics
// 11A80-11A87 - A

// pub fn load_glyph(rom: &[u8]) -> image::GrayImage {
//     todo!();
// }

// ===
use serde::{Serialize, Serializer};
use bitflags::bitflags;

type DateTime = i64;

// shortFrac   16-bit signed fraction
// Fixed   16.16-bit signed fixed-point number
// FWord   16-bit signed integer that describes a quantity in FUnits, the smallest measurable distance in em space.
// uFWord  16-bit unsigned integer that describes a quantity in FUnits, the smallest measurable distance in em space.
// F2Dot14 16-bit signed fixed number with the low 14 bits representing fraction.
// longDateTime    The long internal format of a date in seconds since 12:00 midnight, January 1, 1904. It is represented as a signed 64-bit integer.

// > A table is a sequence of words. Each table must be long aligned and padded with zeroes if necessary.
// https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6.html
// struct Font {
//     cmap: ?, // character to glyph mapping
//     glyf: ?, // glyph data
//     head: ?, // font header
//     hhea: ?, // horizontal header
//     hmtx: ?, // horizontal metrics
//     loca: ?, // index to location
//     maxp: ?, // maximum profile
//     name: ?, // naming
//     post: ?, // PostScript
// }

struct BitmapFont {
    cmap: CMap, // character to glyph mapping
    bhed: Head, // bitmap font header
    bdat: BitmapData, // bitmap data table
    bloc: ?, // bitmap location table
    // hhea: ?, // horizontal header
    // hmtx: ?, // horizontal metrics
    // loca: ?, // index to location
    maxp: ?, // maximum profile
    name: Name, // naming
    // post: ?, // PostScript
}

impl Serialize for BitmapFont {
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

// https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6bdat.html
struct BitmapData {
    version: u32,
    // XXX: u64 is insufficient for _any_ sort of generality,
    // but it might be all i need in this moment.
    glyph_bitmaps: Vec<u64>,
}

impl Serialize for BitmapData {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        // let mut buf = Cursor::new(Vec::new());
        // buf.
        serializer.serialize_bytes(&buf.into_inner())
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

// https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6name.html
struct Name {
    name_records: Vec<NameRecord>,
}

impl Serialize for Name {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        // format
        let buf = vec![0x00, 0x00];
        // number of records
        let len = self.name_records.len() as u16;
        buf.push_be_u16(len);

        // offset to string storage
        // - 12 is sizeof NameRecord
        // - 4 is this byte
        buf.push_be_u16(len * 12 + 4);

        let mut offset = 0;
        let mut str_buffer = Vec::new();
        for record in &self.name_records {
            buf.push_be_u16(record.platform_id);
            buf.push_be_u16(record.encoding_id);
            buf.push_be_u16(record.language_id);
            buf.push_be_u16(record.name_id);
            // TODO: this needs to be UTF-16BE, NOT UTF-8, because we are in hell
            let bytes = record.text.bytes();
            let len = bytes.len() as u16;
            buf.push_be_u16(len);
            buf.push_be_u16(offset);
            offset += len;
            str_buffer.extend(bytes);
        }
        serializer.serialize_bytes(&buf)
    }
}

trait PushBytes {
    fn push_be_u16(&mut self, bytes: u16);
    fn push_le_u16(&mut self, bytes: u16);
    fn push_be_u32(&mut self, bytes: u32);
    fn push_le_u32(&mut self, bytes: u32);
    fn push_be_i16(&mut self, bytes: i16);
    fn push_le_i16(&mut self, bytes: i16);
    fn push_be_i64(&mut self, bytes: i64);
    fn push_le_i64(&mut self, bytes: i64);
}

impl PushBytes for Vec<u8> {
    fn push_be_u16(&mut self, bytes: u16) {
        self.extend(bytes.to_be_bytes());
    }

    fn push_le_u16(&mut self, bytes: u16) {
        self.extend(bytes.to_le_bytes());
    }

    fn push_be_u32(&mut self, bytes: u32) {
        self.extend(bytes.to_be_bytes());
    }

    fn push_le_u32(&mut self, bytes: u32) {
        self.extend(bytes.to_le_bytes());
    }

    fn push_be_i16(&mut self, bytes: i16) {
        self.extend(bytes.to_be_bytes());
    }

    fn push_le_i16(&mut self, bytes: i16) {
        self.extend(bytes.to_le_bytes());
    }

    fn push_be_i64(&mut self, bytes: i64) {
        self.extend(bytes.to_be_bytes());
    }

    fn push_le_i64(&mut self, bytes: i64) {
        self.extend(bytes.to_le_bytes());
    }
}

struct NameRecord {
    platform_id: u16,
    encoding_id: u16,
    language_id: u16,
    name_id: u16,
    text: String
}

// https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6cmap.html
struct CMap {
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
            let subtable: Vec<u8> = record.subtable;
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

// https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6head.html
struct Head {
    version: u32,
    font_revision: u32,
    checksum_adjustment: u32,
    // magic_number: u32,
    flags: u16,
    units_per_em: u16,
    created: DateTime,
    modified: DateTime,
    x_min: i16,
    y_min: i16,
    x_max: i16,
    y_max: i16,
    mac_style: MacStyle,
    lowest_rec_ppem: u16,  // smallest readable size in pixels
    font_direction_hint: i16,
    index_to_loc_format: i16,  // 0 for short offsets, 1 for long
    glyph_data_format: i16,
}

impl Serialize for Head {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut buf = Vec::new();
        buf.push_be_u32(self.version);
        buf.push_be_u32(self.font_revision);
        buf.push_be_u32(self.checksum_adjustment);
        buf.push_be_u32(0x5F0F3CF5);  // magic number
        buf.push_be_u16(self.flags);
        buf.push_be_u16(self.units_per_em);
        buf.push_be_i64(self.created);
        buf.push_be_i64(self.modified);
        buf.push_be_i16(self.x_min);
        buf.push_be_i16(self.y_min);
        buf.push_be_i16(self.x_max);
        buf.push_be_i16(self.y_max);
        buf.push_be_u16(self.mac_style.bits());
        buf.push_be_u16(self.lowest_rec_ppem);
        buf.push_be_i16(self.font_direction_hint);
        buf.push_be_i16(self.index_to_loc_format);
        buf.push_be_i16(self.glyph_data_format);
        buf.push_be_u16(0x0000);
        serializer.serialize_bytes(&buf)
    }
}

bitflags! {
    struct MacStyle: u16 {
        const Bold =      0b000001;
        const Italic =    0b000010;
        const Underline = 0b000100;
        const Outline =   0b001000;
        const Narrow =    0b010000;
        const Extended =  0b100000;
    }
}
