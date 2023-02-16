// https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6name.html
use crate::{FontTable, TableWriter};
use crate::platform::Platform;
use std::io::{self, Write};
use byteorder::{BigEndian, WriteBytesExt};

pub(crate) struct Name {
    name_records: Vec<NameRecord>,
}

impl Name {
    pub fn new() -> Self {
        Name { name_records: Vec::new() }
    }

    pub fn push(&mut self, name_id: u16, text: impl AsRef<str>) {
        self.name_records.push(NameRecord {
            platform: Platform::microsoft_bmp(),
            language_id: ENGLISH_UNITEDSTATES,  // TODO: gross no
            name_id,
            text: text.as_ref().to_string(),
        });
    }
}

impl FontTable for Name {
    const TAG: &'static [u8; 4] = b"name";

    fn write<W: Write>(&self, writer: &mut TableWriter<W>) -> io::Result<()> {
        // format
        writer.write_u16::<BigEndian>(0x0000)?;
        // number of records
        let len = self.name_records.len() as u16;
        writer.write_u16::<BigEndian>(len)?;

        // offset to string storage
        // - 12 is sizeof NameRecord
        // - 6 is this prefix
        writer.write_u16::<BigEndian>(len * 12 + 6)?;

        let mut offset = 0;
        let mut str_buffer = Vec::new();
        for record in &self.name_records {
            let [platform_id, encoding_id] = record.platform.to_bytes();
            writer.write_u16::<BigEndian>(platform_id)?;
            writer.write_u16::<BigEndian>(encoding_id)?;
            writer.write_u16::<BigEndian>(record.language_id)?;
            writer.write_u16::<BigEndian>(record.name_id)?;
            let bytes: Vec<_> = record.text.encode_utf16().collect();
            let len = bytes.len() as u16 * 2;
            writer.write_u16::<BigEndian>(len)?;
            writer.write_u16::<BigEndian>(offset)?;
            offset += len;
            str_buffer.extend(bytes);
        }

        for pair in str_buffer {
            writer.write_u16::<BigEndian>(pair)?;
        }
        Ok(())
    }
}

struct NameRecord {
    platform: Platform,
    language_id: u16,
    name_id: u16,
    text: String,
}

// TODO: these constants could use some types..
// and also.. the rest of them..

// Name IDs
pub const FONT_FAMILY: u16 = 1;

// Microsoft Languages
pub const ENGLISH_UNITEDSTATES: u16 =  0x0409;
