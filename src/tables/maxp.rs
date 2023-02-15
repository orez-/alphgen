use byteorder::{BigEndian, WriteBytesExt};
use crate::{FontTable, TableWriter};
use std::io::{self, Write};

pub(crate) struct MaxP {
    pub version: u32,
    pub num_glyphs: u16,  // the number of glyphs in the font
    pub max_points: u16,  // points in non-compound glyph
    pub max_contours: u16,  // contours in non-compound glyph
    pub max_component_points: u16,  // points in compound glyph
    pub max_component_contours: u16,  // contours in compound glyph
    pub max_zones: u16,  // set to 2
    pub max_twilight_points: u16,  // points used in Twilight Zone (Z0)
    pub max_storage: u16,  // number of Storage Area locations
    pub max_function_defs: u16,  // number of FDEFs
    pub max_instruction_defs: u16,  // number of IDEFs
    pub max_stack_elements: u16,  // maximum stack depth
    pub max_size_of_instructions: u16,  // byte count for glyph instructions
    pub max_component_elements: u16,  // number of glyphs referenced at top level
    pub max_component_depth: u16,  // levels of recursion, set to 0 if font has only simple glyphs
}

impl FontTable for MaxP {
    const TAG: &'static [u8; 4] = b"maxp";

    fn write<W: Write>(&self, writer: &mut TableWriter<W>) -> io::Result<()> {
        writer.write_u32::<BigEndian>(self.version)?;
        writer.write_u16::<BigEndian>(self.num_glyphs)?;
        writer.write_u16::<BigEndian>(self.max_points)?;
        writer.write_u16::<BigEndian>(self.max_contours)?;
        writer.write_u16::<BigEndian>(self.max_component_points)?;
        writer.write_u16::<BigEndian>(self.max_component_contours)?;
        writer.write_u16::<BigEndian>(self.max_zones)?;
        writer.write_u16::<BigEndian>(self.max_twilight_points)?;
        writer.write_u16::<BigEndian>(self.max_storage)?;
        writer.write_u16::<BigEndian>(self.max_function_defs)?;
        writer.write_u16::<BigEndian>(self.max_instruction_defs)?;
        writer.write_u16::<BigEndian>(self.max_stack_elements)?;
        writer.write_u16::<BigEndian>(self.max_size_of_instructions)?;
        writer.write_u16::<BigEndian>(self.max_component_elements)?;
        writer.write_u16::<BigEndian>(self.max_component_depth)?;
        Ok(())
    }
}
