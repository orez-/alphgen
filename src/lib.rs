mod tables;

use std::io::{self, Seek, SeekFrom, Write};
use byteorder::{BigEndian, WriteBytesExt};
use crate::tables::{CMap, Glyf, Head, Name};
use SeekFrom::Start;

// shortFrac   16-bit signed fraction
// Fixed   16.16-bit signed fixed-point number
// FWord   16-bit signed integer that describes a quantity in FUnits, the smallest measurable distance in em space.
// uFWord  16-bit unsigned integer that describes a quantity in FUnits, the smallest measurable distance in em space.
// F2Dot14 16-bit signed fixed number with the low 14 bits representing fraction.
// longDateTime    The long internal format of a date in seconds since 12:00 midnight, January 1, 1904. It is represented as a signed 64-bit integer.

// > A table is a sequence of words. Each table must be long aligned and padded with zeroes if necessary.
// https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6.html
pub struct Font {
    cmap: CMap, // character to glyph mapping
    glyf: Glyf, // glyph data
    head: Head, // font header
    // hhea: ?, // horizontal header
    // hmtx: ?, // horizontal metrics
    // loca: ?, // index to location
    // maxp: ?, // maximum profile
    name: Name, // naming
    // post: ?, // PostScript
}

impl Font {
    pub fn write_to<W: Write + Seek>(&self, writer: &mut W) -> io::Result<()> {
        // TODO: I don't know what the best way to represent these tables is,
        // but hardcoding the length like this is almost surely Not It.
        let table_count = 4;

        // Header
        // u32 - magic
        writer.write_all(&[0x00, 0x01, 0x00, 0x00])?;
        writer.write_u16::<BigEndian>(table_count)?;
        // u16 - number of tables
        // u16 - search range
        // u16 - entry selector
        // u16 - range shift
        let mut record_ptr = 12;
        let mut table_ptr = 12 + table_count as u32 * 16;
        writer.seek(Start(table_ptr as u64))?;
        // Table Records
        self.write_table(writer, &self.cmap)?;
        self.write_table(writer, &self.glyf)?;
        self.write_table(writer, &self.head)?;
        self.write_table(writer, &self.name)?;
        Ok(())
    }

    fn write_table<W: Write + Seek, T: FontTable>(&self, writer: &mut W, table: &T) -> io::Result<()> {
        // TODO: ughgauhfdodidisfb
        // I don't want to `&mut`-pass these two ↓↓↓ knuckleheads to maintain em,
        // I don't want to keep em attached to the `Font`.
        // Can't sneak em in by converting `write_table` to a closure in `write_to`:
        // closures can't have generics (which.. makes sense. something about closures
        // implicitly capturing + borrowing ownership of variables feels incompatible with
        // compiling multiple versions for the different concrete types).
        // Could associate em with a `Write+Seek` adapter?? Idk this feels excessive.
        // Maybe on a `Font` adapter???? That just feels silly.
        // Regardless, they're just broken currently, so.. figure that out.
        let mut record_ptr = 0;
        let mut table_ptr = 0;

        let mut twriter = TableWriter::new(writer);
        table.write(&mut twriter)?;
        let (checksum, length) = twriter.finalize()?;
        writer.seek(Start(record_ptr))?;
        writer.write_all(T::TAG)?;
        writer.write_u32::<BigEndian>(checksum)?;
        writer.write_u32::<BigEndian>(table_ptr)?;
        writer.write_u32::<BigEndian>(length)?;
        record_ptr += 8;
        table_ptr += length;
        writer.seek(Start(table_ptr as u64))?;
        Ok(())
    }
}

trait FontTable {
    const TAG: &'static [u8; 4];
    fn write<W: Write>(&self, writer: &mut TableWriter<W>) -> io::Result<()>;
}

struct TableWriter<'a, W: Write> {
    writer: &'a mut W,
    checksum: u32,
    in_progress_word: [u8; 4],
    length: usize,
}

impl<'a, W: Write> TableWriter<'a, W> {
    fn new(writer: &'a mut W) -> TableWriter<'a, W> {
        Self {
            writer,
            checksum: 0,
            in_progress_word: [0, 0, 0, 0],
            length: 0,
        }
    }

    fn finalize(mut self) -> io::Result<(u32, u32)> {
        let count = (4 - self.length % 4) % 4;
        if count != 0 {
            self.write_all(&vec![0; count])?;
        }
        Ok((self.checksum, self.length as u32))
    }
}

impl<'a, W: Write> Write for TableWriter<'a, W> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let out = self.writer.write(bytes)?;
        if bytes.is_empty() { return Ok(out); }
        let bytes = &bytes[..out];

        // front
        let len = self.length % 4;
        let to_take = bytes.len().min(4 - len);
        let (front, bytes) = bytes.split_at(to_take);
        if len != 0 {
            self.in_progress_word[len..][..to_take]
                .copy_from_slice(front);
            self.length += to_take;
            if len + to_take < 4 {
                return Ok(out);
            }
            let word = u32::from_be_bytes(self.in_progress_word);
            self.checksum = self.checksum.wrapping_add(word);
        }

        // middle
        // XXX: array_chunks is nightly as of 1.67
        let mut chunks = bytes.chunks_exact(4);
        for chunk in chunks.by_ref() {
            let chunk = chunk.try_into().unwrap();
            let word = u32::from_be_bytes(chunk);
            self.checksum = self.checksum.wrapping_add(word);
        }

        // end
        let rem = chunks.remainder();
        self.in_progress_word[..rem.len()]
            .copy_from_slice(rem);

        Ok(out)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

// Bit-aligned bitmap data padded to byte boundaries.

// Optional tables
// 'cvt '  control value
// 'fpgm'  font program
// 'hdmx'  horizontal device metrics
// 'kern'  kerning
// 'OS/2'  OS/2
// 'prep'  control value program

type Bitmap<'a> = &'a [u8];

pub fn bitmap_font<'a, G, L>(width: usize, height: usize, glyphs: G, ligatures: L) -> Font
where
    G: IntoIterator<Item=(char, Bitmap<'a>)>,
    L: IntoIterator<Item=(&'a str, Bitmap<'a>)>,
{
    todo!();
}
