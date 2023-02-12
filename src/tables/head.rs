// https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6head.html
use crate::{FontTable, TableWriter};
use bitflags::bitflags;
use byteorder::{BigEndian, WriteBytesExt};
use std::io::{self, Write};

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

impl FontTable for Head {
    const TAG: &'static [u8; 4] = b"head";

    fn write<W: Write>(&self, writer: &mut TableWriter<W>) -> io::Result<()> {
        writer.write_u32::<BigEndian>(self.version)?;
        writer.write_u32::<BigEndian>(self.font_revision)?;
        writer.write_u32::<BigEndian>(self.checksum_adjustment)?;
        writer.write_u32::<BigEndian>(0x5F0F3CF5)?;  // magic number
        writer.write_u16::<BigEndian>(self.flags)?;
        writer.write_u16::<BigEndian>(self.units_per_em)?;
        writer.write_i64::<BigEndian>(self.created)?;
        writer.write_i64::<BigEndian>(self.modified)?;
        writer.write_i16::<BigEndian>(self.x_min)?;
        writer.write_i16::<BigEndian>(self.y_min)?;
        writer.write_i16::<BigEndian>(self.x_max)?;
        writer.write_i16::<BigEndian>(self.y_max)?;
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
    struct MacStyle: u16 {
        const Bold =      0b000001;
        const Italic =    0b000010;
        const Underline = 0b000100;
        const Outline =   0b001000;
        const Narrow =    0b010000;
        const Extended =  0b100000;
    }
}
