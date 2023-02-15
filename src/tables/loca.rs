// https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6loca.html
use crate::{FontTable, TableWriter};
use std::io::{self, Write};
use byteorder::{BigEndian, WriteBytesExt};

pub(crate) struct Loca {
}

impl FontTable for Loca {
    const TAG: &'static [u8; 4] = b"loca";

    fn write<W: Write>(&self, writer: &mut TableWriter<W>) -> io::Result<()> {
        Ok(())
    }
}
