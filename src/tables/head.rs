// https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6head.html
#![allow(non_upper_case_globals)]

use crate::{FontTable, Rect, TableWriter};
use crate::time;
use bitflags::bitflags;
use byteorder::{BigEndian, WriteBytesExt};
use std::io::{self, Write};

pub(crate) struct Head {
    version: u32,
    font_revision: u32,
    checksum_adjustment: u32,
    flags: Flags,
    units_per_em: u16,
    created: time::DateTime,
    modified: time::DateTime,
    rect: Rect,
    mac_style: MacStyle,
    lowest_rec_ppem: u16,  // smallest readable size in pixels
    font_direction_hint: i16,
    pub index_to_loc_format: i16,  // 0 for short offsets, 1 for long
    glyph_data_format: i16,
}

impl Head {
    pub(crate) fn new() -> Self {
        let now = time::now();
        // XXX: should this be autogenerated???????
        let rect = Rect {
            x_min: 0,
            y_min: 0,
            x_max: 8,
            y_max: 8,
        };

        Head {
            version: 0x00010000,
            font_revision: 0x00010000,
            checksum_adjustment: 0,
            flags: Flags::INTEGER_SCALING,
            units_per_em: 16, // XXX: this one actually matters
            created: now,
            modified: now,
            rect,
            mac_style: MacStyle::empty(),
            lowest_rec_ppem: 8,  // XXX: probably right for our purposes, but bad hardcode
            font_direction_hint: 1,  // XXX :(
            index_to_loc_format: 1,  // XXX
            glyph_data_format: 0,  // ..???
        }
    }
}

impl FontTable for Head {
    const TAG: &'static [u8; 4] = b"head";

    fn write<W: Write>(&self, writer: &mut TableWriter<W>) -> io::Result<()> {
        writer.write_u32::<BigEndian>(self.version)?;
        writer.write_u32::<BigEndian>(self.font_revision)?;
        writer.write_u32::<BigEndian>(self.checksum_adjustment)?;
        writer.write_u32::<BigEndian>(0x5F0F3CF5)?;  // magic number
        writer.write_u16::<BigEndian>(self.flags.bits)?;
        writer.write_u16::<BigEndian>(self.units_per_em)?;
        writer.write_i64::<BigEndian>(self.created)?;
        writer.write_i64::<BigEndian>(self.modified)?;
        self.rect.write(writer)?;
        writer.write_u16::<BigEndian>(self.mac_style.bits())?;
        writer.write_u16::<BigEndian>(self.lowest_rec_ppem)?;
        writer.write_i16::<BigEndian>(self.font_direction_hint)?;
        writer.write_i16::<BigEndian>(self.index_to_loc_format)?;
        writer.write_i16::<BigEndian>(self.glyph_data_format)?;
        writer.write_u16::<BigEndian>(0x0000)?;
        Ok(())
    }
}

bitflags! {
    struct Flags: u16 {
        const INTEGER_SCALING = 1 << 3;
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
