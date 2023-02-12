// https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6glyf.html
use crate::{FontTable, TableWriter};
use std::io::{self, Write};

pub(crate) struct Glyf {
}

impl FontTable for Glyf {
    const TAG: &'static [u8; 4] = b"glyf";

    fn write<W: Write>(&self, writer: &mut TableWriter<W>) -> io::Result<()> {
        Ok(())
    }
}
