// https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6head.html
use bitflags::bitflags;
use crate::push_bytes::PushBytes;
use serde::{Serialize, Serializer};

type DateTime = i64;

pub(crate) struct Head {
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
