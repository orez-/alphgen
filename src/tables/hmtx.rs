// https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6hmtx.html
use crate::{FontTable, TableWriter};
use std::io::{self, Write};
use byteorder::{BigEndian, WriteBytesExt};

pub(crate) struct HMtx {
    horizontal_metrics: Vec<HorizontalMetric>,
}

impl HMtx {
    pub fn monospace(advance_width: u16, lsbs: Vec<i16>) -> Self {
        let horizontal_metrics: Vec<_> =
            lsbs.into_iter().map(|left_side_bearing|
                HorizontalMetric { advance_width, left_side_bearing }
            ).collect();
        HMtx { horizontal_metrics }
    }

    pub fn num_of_long_hor_metrics(&self) -> usize {
        let advance_width = self.horizontal_metrics.last()
            .expect("horizontal_metrics may not be empty")
            .advance_width;
        self.horizontal_metrics.iter()
            .rposition(|hmtx| hmtx.advance_width != advance_width)
            .map_or(1, |x| x + 2)
    }
}

impl FontTable for HMtx {
    const TAG: &'static [u8; 4] = b"hmtx";
    fn write<W: Write>(&self, writer: &mut TableWriter<W>) -> io::Result<()> {
        let idx = self.num_of_long_hor_metrics();
        let (both, lsb) = self.horizontal_metrics.split_at(idx);
        for hmtx in both {
            writer.write_u16::<BigEndian>(hmtx.advance_width)?;
            writer.write_i16::<BigEndian>(hmtx.left_side_bearing)?;
        }
        for hmtx in lsb {
            writer.write_i16::<BigEndian>(hmtx.left_side_bearing)?;
        }
        Ok(())
    }
}

struct HorizontalMetric {
    advance_width: u16,
    left_side_bearing: i16,
}
