// https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6glyf.html
use byteorder::{BigEndian, WriteBytesExt};
use bitflags::bitflags;
use crate::{FontTable, Rect, TableWriter};
use crate::sprite::Sprite;
use crate::itertools::split_when;
use std::io::{self, Cursor, Write};

pub(crate) struct Glyf {
    glyphs: Vec<Glyph>,
}

impl FontTable for Glyf {
    const TAG: &'static [u8; 4] = b"glyf";

    fn write<W: Write>(&self, writer: &mut TableWriter<W>) -> io::Result<()> {
        // XXX: I think the `loca` table needs these offsets.
        // I will worry about this later.
        for glyph in &self.glyphs {
            glyph.write(writer)?;
        }
        Ok(())
    }
}

struct Glyph {
    rect: Rect,
    glyph_data: GlyphData,
}

impl Glyph {
    fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
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
}

impl From<&Sprite> for Glyph {
    fn from(sprite: &Sprite) -> Self {
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
    let Repeat = GlyphFlags::Repeat;
    let mut out = Vec::new();
    for group in split_when(flags, |a, b| a != b) {
        let flag = group[0];
        let len = group.len();
        let full_repeats = len / 256;
        let stragglers = (len % 256) as u8;
        for _ in 0..full_repeats {
            out.push((flag | Repeat).bits);
            out.push(0xff);
        }
        match stragglers {
            0 => (),
            1 => out.push(flag.bits),
            c => {
                out.push((flag | Repeat).bits);
                out.push(c - 1);
            }
        }
    }
    out
}
