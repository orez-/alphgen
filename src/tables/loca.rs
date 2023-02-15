// https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6loca.html
use crate::{FontTable, TableWriter};
use std::io::{self, Write};
use byteorder::{BigEndian, WriteBytesExt};

pub(crate) struct Loca {
    offsets: Vec<usize>,
}

impl Loca {
    pub fn from(offsets: Vec<usize>) -> Self {
        Self { offsets }
    }

    pub fn needs_long(&self) -> bool {
        let longest = u16::MAX as usize;
        matches!(self.offsets.last(), Some(&idx) if idx > longest)
    }
}

impl FontTable for Loca {
    const TAG: &'static [u8; 4] = b"loca";

    fn write<W: Write>(&self, writer: &mut TableWriter<W>) -> io::Result<()> {
        if self.needs_long() {
            for &off in &self.offsets {
                writer.write_u32::<BigEndian>(off.try_into().unwrap())?;
            }
        } else {
            for &off in &self.offsets {
                writer.write_u16::<BigEndian>(off as u16)?;
            }
        }
        Ok(())
    }
}
