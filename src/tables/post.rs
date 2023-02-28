// https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6post.html
use crate::{FontTable, GlyphId, TableWriter};
use std::io::{self, Write};
use std::iter::repeat;
use byteorder::{BigEndian, WriteBytesExt};

const NOT_DEF: GlyphId = GlyphId(0);

pub(crate) struct Post {
    italic_angle: u32,
    underline_position: i16,
    underline_thickness: i16,
    is_monospace: bool,
    min_mem_type42: u32,
    max_mem_type42: u32,
    min_mem_type1: u32,
    max_mem_type1: u32,
    format: PostFormat,
}

impl Post {
    pub fn from_ascii_order(order: &[char], count: usize) -> Self {
        let diff = count.checked_sub(order.len()+1)
            .expect("count must be > order.len()");
        let glyphs = order.into_iter()
            .map(|&c| to_macintosh(c).unwrap_or(NOT_DEF));
        let names: Vec<_> = [NOT_DEF].into_iter().chain(glyphs)
            .chain(repeat(NOT_DEF).take(diff))
            .map(GlyphName::Preset)
            .collect();
        assert_eq!(names.len(), count);
        let format = PostFormat::Format2 { names };

        // TODO: don't hardcode the first four here.
        // the mem fields seem optional
        Post {
            italic_angle: 0,
            underline_position: 0,
            underline_thickness: 1,
            is_monospace: true,
            min_mem_type42: 0,
            max_mem_type42: 0,
            min_mem_type1: 0,
            max_mem_type1: 0,
            format,
        }
    }
}

impl FontTable for Post {
    const TAG: &'static [u8; 4] = b"post";

    fn write<W: Write>(&self, writer: &mut TableWriter<W>) -> io::Result<()> {
        writer.write_u32::<BigEndian>(self.format.format())?;
        writer.write_u32::<BigEndian>(self.italic_angle)?;
        writer.write_i16::<BigEndian>(self.underline_position)?;
        writer.write_i16::<BigEndian>(self.underline_thickness)?;
        writer.write_u32::<BigEndian>(self.is_monospace as u32)?;
        writer.write_u32::<BigEndian>(self.min_mem_type42)?;
        writer.write_u32::<BigEndian>(self.max_mem_type42)?;
        writer.write_u32::<BigEndian>(self.min_mem_type1)?;
        writer.write_u32::<BigEndian>(self.max_mem_type1)?;
        self.format.write(writer)?;
        Ok(())
    }
}

enum PostFormat {
    Format2 {
        names: Vec<GlyphName>,
    }
}

enum GlyphName {
    Preset(GlyphId),
    Custom(String),
}

impl PostFormat {
    fn format(&self) -> u32 {
        match self {
            PostFormat::Format2 { .. } => 0x00020000,
        }
    }

    fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        match self {
            PostFormat::Format2 { names } => {
                writer.write_u16::<BigEndian>(names.len() as u16)?;
                let mut string_bytes = Vec::new();
                let mut str_offset = 258;
                for name in names {
                    match name {
                        GlyphName::Preset(p) =>
                            writer.write_u16::<BigEndian>(p.0)?,
                        GlyphName::Custom(s) => {
                            writer.write_u16::<BigEndian>(str_offset)?;
                            string_bytes.extend(to_pascal_string(s));
                        }
                    }
                }
                writer.write_all(&string_bytes)?;
            }
        }
        Ok(())
    }
}

// the docs just say "Pascal string".
// gonna assume this is what they mean
fn to_pascal_string(s: &str) -> Vec<u8> {
    let string: Vec<u16> = s.encode_utf16().collect();
    let len = string.len() as u16;
    let mut out = Vec::new();
    out.extend(len.to_be_bytes());
    for c in string {
        out.extend(c.to_be_bytes());
    }
    out
}

fn to_macintosh(c: char) -> Option<GlyphId> {
    let lookup =
        " !\"#$%&'()*+,-./0123456789:;<=>?@\
        ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`\
        abcdefghijklmnopqrstuvwxyz{|}~\
        ÄÅÇÉÑÖÜáàâäãåçéèêëíìîïñóòôöõúùûü\
        †˚¢£§•¶ß®©™´¨≠åÆØ∞±≤≥¥µ∂∑∏π∫ªºΩæø\
        ¿¡¬√ƒ≈Δ«»… ÁÃÕŒœ–—“”‘’÷◊ÿŸ⁄¤‹›ﬁﬂ\
        ‡·‚„‰ÂÊÁËÈÍÎÏÌÓÔÒÚÛÙı";
    lookup.chars()
        .position(|x| x == c)
        .map(|x| GlyphId(x as u16 + 3))
}
