// https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6OS2.html
use byteorder::{BigEndian, WriteBytesExt};
use crate::{FontTable, TableWriter};
use std::io::{self, Write};

pub(crate) enum Os2 {
    Version5(Os2V5),
}

impl Default for Os2 {
    fn default() -> Self {
        let os2 = Os2V5 {
            x_avg_char_width: 8,
            us_weight_class: 400,
            us_width_class: 5,
            fs_type: 0,
            y_subscript_x_size: 8,
            y_subscript_y_size: 8,
            y_subscript_x_offset: 0,
            y_subscript_y_offset: 0,
            y_superscript_x_size: 8,
            y_superscript_y_size: 8,
            y_superscript_x_offset: 0,
            y_superscript_y_offset: 0,
            y_strikeout_size: 1,
            y_strikeout_position: 4,
            s_family_class: 0,
            panose: [0; 10],
            ul_unicode_range: 0,
            ach_vend_id: *b"    ",
            fs_selection: 0,
            us_first_char_index: 0,
            us_last_char_index: 128,
            s_typo_ascender: 0,
            s_typo_descender: 0,
            s_typo_line_gap: 0,
            us_win_ascent: 8,
            us_win_descent: 0,
            ul_code_page_range: 0,
            sx_height: 5,
            s_cap_height: 6,
            us_default_char: 0,
            us_break_char: 0x20,
            us_max_context: 2,
            us_lower_optical_point_size: 0,
            us_upper_optical_point_size: 0xFFFF,
        };
        Os2::Version5(os2)
    }
}

pub(crate) struct Os2V5 {
    x_avg_char_width: i16,
    us_weight_class: u16,
    us_width_class: u16,
    fs_type: u16,
    y_subscript_x_size: i16,
    y_subscript_y_size: i16,
    y_subscript_x_offset: i16,
    y_subscript_y_offset: i16,
    y_superscript_x_size: i16,
    y_superscript_y_size: i16,
    y_superscript_x_offset: i16,
    y_superscript_y_offset: i16,
    y_strikeout_size: i16,
    y_strikeout_position: i16,
    s_family_class: i16,
    panose: [u8; 10],
    ul_unicode_range: u128,
    ach_vend_id: [u8; 4],
    fs_selection: u16,
    us_first_char_index: u16,
    us_last_char_index: u16,
    s_typo_ascender: i16,
    s_typo_descender: i16,
    s_typo_line_gap: i16,
    us_win_ascent: u16,
    us_win_descent: u16,
    ul_code_page_range: u64,
    sx_height: i16,
    s_cap_height: i16,
    us_default_char: u16,
    us_break_char: u16,
    us_max_context: u16,
    us_lower_optical_point_size: u16,
    us_upper_optical_point_size: u16,
}

impl FontTable for Os2 {
    const TAG: &'static [u8; 4] = b"OS/2";

    fn write<W: Write>(&self, writer: &mut TableWriter<W>) -> io::Result<()> {
        match self {
            Os2::Version5(this) => {
                writer.write_u16::<BigEndian>(0x05)?;  // version
                writer.write_i16::<BigEndian>(this.x_avg_char_width)?;
                writer.write_u16::<BigEndian>(this.us_weight_class)?;
                writer.write_u16::<BigEndian>(this.us_width_class)?;
                writer.write_u16::<BigEndian>(this.fs_type)?;
                writer.write_i16::<BigEndian>(this.y_subscript_x_size)?;
                writer.write_i16::<BigEndian>(this.y_subscript_y_size)?;
                writer.write_i16::<BigEndian>(this.y_subscript_x_offset)?;
                writer.write_i16::<BigEndian>(this.y_subscript_y_offset)?;
                writer.write_i16::<BigEndian>(this.y_superscript_x_size)?;
                writer.write_i16::<BigEndian>(this.y_superscript_y_size)?;
                writer.write_i16::<BigEndian>(this.y_superscript_x_offset)?;
                writer.write_i16::<BigEndian>(this.y_superscript_y_offset)?;
                writer.write_i16::<BigEndian>(this.y_strikeout_size)?;
                writer.write_i16::<BigEndian>(this.y_strikeout_position)?;
                writer.write_i16::<BigEndian>(this.s_family_class)?;
                writer.write_all(&this.panose)?;
                writer.write_u128::<BigEndian>(this.ul_unicode_range)?;
                writer.write_all(&this.ach_vend_id)?;
                writer.write_u16::<BigEndian>(this.fs_selection)?;
                writer.write_u16::<BigEndian>(this.us_first_char_index)?;
                writer.write_u16::<BigEndian>(this.us_last_char_index)?;
                writer.write_i16::<BigEndian>(this.s_typo_ascender)?;
                writer.write_i16::<BigEndian>(this.s_typo_descender)?;
                writer.write_i16::<BigEndian>(this.s_typo_line_gap)?;
                writer.write_u16::<BigEndian>(this.us_win_ascent)?;
                writer.write_u16::<BigEndian>(this.us_win_descent)?;
                writer.write_u64::<BigEndian>(this.ul_code_page_range)?;
                writer.write_i16::<BigEndian>(this.sx_height)?;
                writer.write_i16::<BigEndian>(this.s_cap_height)?;
                writer.write_u16::<BigEndian>(this.us_default_char)?;
                writer.write_u16::<BigEndian>(this.us_break_char)?;
                writer.write_u16::<BigEndian>(this.us_max_context)?;
                writer.write_u16::<BigEndian>(this.us_lower_optical_point_size)?;
                writer.write_u16::<BigEndian>(this.us_upper_optical_point_size)?;
            }
        }
        Ok(())
    }
}
