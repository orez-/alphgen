// https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6glyf.html
#![allow(non_upper_case_globals)]

use byteorder::{BigEndian, WriteBytesExt};
use bitflags::bitflags;
use crate::{FontTable, Rect, TableWriter};
use crate::sprite::Sprite;
use crate::tables::{Loca, MaxP};
use crate::itertools::split_when;
use crate::writeutils::CountWriter;
use std::io::{self, Cursor, Write};

pub(crate) struct Glyf {
    glyphs: Vec<Glyph>,
}

impl FontTable for Glyf {
    const TAG: &'static [u8; 4] = b"glyf";

    fn write<W: Write>(&self, writer: &mut TableWriter<W>) -> io::Result<()> {
        for glyph in &self.glyphs {
            glyph.write(writer)?;
        }
        Ok(())
    }
}

impl Glyf {
    pub(crate) fn from<I: IntoIterator<Item=G>, G: Into<Glyph>>(it: I) -> Self {
        let glyphs: Vec<Glyph> = it.into_iter().map(|x| x.into()).collect();
        Glyf { glyphs }
    }

    // XXX: i hate it!
    // i think probably glyf + loca + cmap should be one struct that serializes
    // into the three tables. Having to maintain invariants between these three
    // tables sounds baaaaa-aad. I do not want it.
    pub fn generate_loca(&self) -> Loca {
        let mut offsets = vec![0];
        let mut writer = CountWriter::sink();
        for glyph in &self.glyphs {
            glyph.write(&mut writer)
                .expect("we're not writing it anywhere");
            offsets.push(writer.count());
        }
        Loca::from(offsets)
    }

    fn max_aspect<F: Fn(&Glyph) -> usize>(&self, f: F) -> u16 {
        self.glyphs.iter()
            .map(f)
            .max()
            .unwrap_or(0) as u16
    }

    /// The maxComponentDepth refers to the number of levels of recursion used in constructing
    /// the most complex compound glyph. The maximum legal value for maxComponentDepth is 16.
    /// If there are no components within components, all compound glyphs can be deemed simple
    /// and this field can be set to the value one.
    fn max_component_depth(&self) -> u16 {
        1
    }

    pub fn generate_maxp(&self) -> MaxP {
        MaxP {
            version: 0x00010000,
            num_glyphs: self.count_glyphs() as u16,
            max_points: self.max_aspect(Glyph::point_count),
            max_contours: self.max_aspect(Glyph::contour_count),
            max_component_points: self.max_aspect(Glyph::component_point_count),
            max_component_contours: self.max_aspect(Glyph::component_contour_count),
            max_zones: 2,
            max_twilight_points: 0,
            max_storage: 0,
            max_function_defs: 0,
            max_instruction_defs: 0,
            max_stack_elements: 0,
            max_size_of_instructions: self.max_aspect(Glyph::instruction_byte_count),
            max_component_elements: 1,
            max_component_depth: self.max_component_depth(),
        }
    }

    pub fn count_glyphs(&self) -> usize {
        self.glyphs.len()
    }
}

pub(crate) struct Glyph {
    rect: Rect,
    glyph_data: GlyphData,
}

impl Glyph {
    fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        let mut writer = CountWriter::from(writer);
        let writer = &mut writer;
        match &self.glyph_data {
            GlyphData::Simple { instructions, contours } => {
                let contour_count = contours.len() as u16;
                writer.write_u16::<BigEndian>(contour_count)?;
                self.rect.write(writer)?;

                // endPtsOfContours[n]
                let mut pt_count = 0;
                for contour in contours {
                    pt_count += contour.len() as u16;
                    writer.write_u16::<BigEndian>(pt_count - 1)?;
                }

                // instructions
                let instruction_len = instructions.len() as u16;
                writer.write_u16::<BigEndian>(instruction_len)?;
                writer.write_all(&instructions)?;

                // flags + coords
                let mut x = 0;
                let mut y = 0;
                let mut flags = Vec::new();
                let mut dxs = Cursor::new(Vec::new());
                let mut dys = Cursor::new(Vec::new());
                for contour in contours {
                    for pt in contour {
                        let mut flag = GlyphFlags::empty();
                        if pt.on_curve {
                            flag |= GlyphFlags::OnCurve;
                        }

                        let dx: i16 = (pt.x - x).try_into().unwrap();
                        x = pt.x;
                        flag |= Glyph::write_dx(&mut dxs, dx)?;

                        let dy: i16 = (pt.y - y).try_into().unwrap();
                        y = pt.y;
                        flag |= Glyph::write_dy(&mut dys, dy)?;

                        flags.push(flag);
                    }
                }

                let flags = compress_flags(&flags);
                writer.write_all(&flags)?;
                writer.write_all(&dxs.into_inner())?;
                writer.write_all(&dys.into_inner())?;
            }
        }
        // each glyph must be u16-aligned
        if writer.count() % 2 == 1 {
            writer.write_all(&[0x00])?;
        }
        Ok(())
    }

    fn write_dx<W: Write>(dxs: &mut W, dx: i16) -> io::Result<GlyphFlags> {
        let mut flag = GlyphFlags::empty();
        match dx.abs().try_into() {
            Ok(0_u8) => {
                // If the x-short Vector bit is not set, and this bit is set, then
                // the current x-coordinate is the same as the previous
                // x-coordinate.
                flag |= GlyphFlags::XMod;
            }
            Ok(byte) => {
                // If the x-Short Vector bit is set, this bit describes the sign
                // of the value, with a value of 1 equalling positive and a zero
                // value negative.
                flag |= GlyphFlags::XShort;
                if dx > 0 {
                    flag |= GlyphFlags::XMod;
                }
                dxs.write(&[byte])?;
            }
            Err(_) => {
                // If the x-short Vector bit is not set, and this bit is not set,
                // the current x-coordinate is a signed 16-bit delta vector.
                // In this case, the delta vector is the change in x
                dxs.write_i16::<BigEndian>(dx)?;
            }
        }
        Ok(flag)
    }

    fn write_dy<W: Write>(dys: &mut W, dy: i16) -> io::Result<GlyphFlags> {
        let mut flag = GlyphFlags::empty();
        match dy.abs().try_into() {
            Ok(0_u8) => {
                // If the y-short Vector bit is not set, and this bit is set, then
                // the current y-coordinate is the same as the previous
                // y-coordinate.
                flag |= GlyphFlags::YMod;
            }
            Ok(byte) => {
                // If the y-Short Vector bit is set, this bit describes the sign
                // of the value, with a value of 1 equalling positive and a zero
                // value negative.
                flag |= GlyphFlags::YShort;
                if dy > 0 {
                    flag |= GlyphFlags::YMod;
                }
                dys.write(&[byte])?;
            }
            Err(_) => {
                // If the y-short Vector bit is not set, and this bit is not set,
                // the current y-coordinate is a signed 16-bit delta vector.
                // In this case, the delta vector is the change in y
                dys.write_i16::<BigEndian>(dy)?;
            }
        }
        Ok(flag)
    }

    /// Points in non-compound glyph
    fn point_count(&self) -> usize {
        match &self.glyph_data {
            GlyphData::Simple { contours, .. } => contours.iter().map(|c| c.len()).sum(),
        }
    }

    /// Contours in non-compound glyph
    fn contour_count(&self) -> usize {
        match &self.glyph_data {
            GlyphData::Simple { contours, .. } => contours.len(),
        }
    }

    /// Points in compound glyph
    fn component_point_count(&self) -> usize {
        match &self.glyph_data {
            GlyphData::Simple { .. } => 0,
        }
    }

    /// Contours in compound glyph
    fn component_contour_count(&self) -> usize {
        match &self.glyph_data {
            GlyphData::Simple { .. } => 0,
        }
    }

    fn instruction_byte_count(&self) -> usize {
        match &self.glyph_data {
            GlyphData::Simple { instructions, .. } => instructions.len(),
        }
    }
}

impl From<Sprite> for Glyph {
    fn from(sprite: Sprite) -> Self {
        let contours = sprite.find_contours();
        let glyph_data = GlyphData::Simple {
            instructions: Vec::new(),
            contours: into_contours(&contours),
        };
        let rect = Rect {
            x_min: 0,
            y_min: 0,
            x_max: sprite.width().try_into().unwrap(),
            y_max: sprite.height().try_into().unwrap(),
        };
        Glyph { rect, glyph_data }
    }
}

fn into_contours(contours: &[Vec<(usize, usize)>]) -> Vec<Contour> {
    contours.into_iter().map(|contour| {
        contour.into_iter().map(|&(x, y)| Coordinate {
            x: x.try_into().unwrap(),
            y: y.try_into().unwrap(),
            on_curve: true,
        }).collect()
    }).collect()
}

enum GlyphData {
    Simple {
        instructions: Vec<u8>, // XXX: ???????
        contours: Vec<Contour>,
    },
    // Compound(Vec<>),
}

type Contour = Vec<Coordinate>;

struct Coordinate {
    x: i64,
    y: i64,
    on_curve: bool,
}

bitflags! {
    struct GlyphFlags: u8 {
        const OnCurve = 0b000001;
        const XShort =  0b000010;
        const YShort =  0b000100;
        const Repeat =  0b001000;
        const XMod =    0b010000;
        const YMod =    0b100000;
    }
}

bitflags! {
    struct ComponentFlags: u16 {
        const ARG_1_AND_2_ARE_WORDS = 1 << 0;
        const ARGS_ARE_XY_VALUES = 1 << 1;
        const ROUND_XY_TO_GRID = 1 << 2;
        const WE_HAVE_A_SCALE = 1 << 3;
        // bit 4 is obsolete
        const MORE_COMPONENTS = 1 << 5;
        const WE_HAVE_AN_X_AND_Y_SCALE = 1 << 6;
        const WE_HAVE_A_TWO_BY_TWO = 1 << 7;
        const WE_HAVE_INSTRUCTIONS = 1 << 8;
        const USE_MY_METRICS = 1 << 9;
        const OVERLAP_COMPOUND = 1 << 10;
    }
}

fn compress_flags(flags: &[GlyphFlags]) -> Vec<u8> {
    let repeat = GlyphFlags::Repeat;
    let mut out = Vec::new();
    for group in split_when(flags, |a, b| a != b) {
        let flag = group[0];
        let len = group.len();
        let full_repeats = len / 256;
        let stragglers = (len % 256) as u8;
        for _ in 0..full_repeats {
            out.push((flag | repeat).bits);
            out.push(0xff);
        }
        match stragglers {
            0 => (),
            1 => out.push(flag.bits),
            c => {
                out.push((flag | repeat).bits);
                out.push(c - 1);
            }
        }
    }
    out
}
