use byteorder::{BigEndian, WriteBytesExt};
use crate::{FontTable, TableWriter};
use std::io::{self, Write};

pub(crate) struct HHea {
    ascent: i16,  //  Distance from baseline of highest ascender
    descent: i16,  // Distance from baseline of lowest descender
    line_gap: i16,  // typographic line gap
    advance_width_max: u16,  // must be consistent with horizontal metrics
    min_left_side_bearing: i16,  // must be consistent with horizontal metrics
    min_right_side_bearing: i16,  // must be consistent with horizontal metrics
    x_max_extent: i16,  // max(lsb + (xMax-xMin))
    caret_slope_rise: i16,  // used to calculate the slope of the caret (rise/run) set to 1 for vertical caret
    caret_slope_run: i16,  // 0 for vertical
    caret_offset: i16,  // set value to 0 for non-slanted fonts
    metric_data_format: i16,  // ??
    pub num_of_long_hor_metrics: u16,  // number of advance widths in metrics table
}

impl HHea {
    pub fn new() -> Self {
        HHea {
            ascent: 0,
            descent: 0,
            line_gap: 0,
            advance_width_max: 8,
            min_left_side_bearing: 0,
            min_right_side_bearing: 8,
            x_max_extent: 16,
            caret_slope_rise: 1,
            caret_slope_run: 0,
            caret_offset: 0,
            metric_data_format: 0,
            num_of_long_hor_metrics: 0,  // illegal value, must be overwritten
        }
    }
}

impl FontTable for HHea {
    const TAG: &'static [u8; 4] = b"hhea";
    fn write<W: Write>(&self, writer: &mut TableWriter<W>) -> io::Result<()> {
        writer.write_u32::<BigEndian>(0x00010000)?;  // version
        writer.write_i16::<BigEndian>(self.ascent)?;
        writer.write_i16::<BigEndian>(self.descent)?;
        writer.write_i16::<BigEndian>(self.line_gap)?;
        writer.write_u16::<BigEndian>(self.advance_width_max)?;
        writer.write_i16::<BigEndian>(self.min_left_side_bearing)?;
        writer.write_i16::<BigEndian>(self.min_right_side_bearing)?;
        writer.write_i16::<BigEndian>(self.x_max_extent)?;
        writer.write_i16::<BigEndian>(self.caret_slope_rise)?;
        writer.write_i16::<BigEndian>(self.caret_slope_run)?;
        writer.write_i16::<BigEndian>(self.caret_offset)?;
        writer.write_u64::<BigEndian>(0)?;  // reserved
        writer.write_i16::<BigEndian>(self.metric_data_format)?;
        writer.write_u16::<BigEndian>(self.num_of_long_hor_metrics)?;
        Ok(())
    }
}
