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
        self.name_records.push(NameRecord {
            platform: Platform::macintosh_roman(),
            language_id: 0,  // TODO: gross no
            name_id,
            text: text.as_ref().to_string(),
        });
    }
}

impl FontTable for Name {
    const TAG: &'static [u8; 4] = b"name";

    fn write<W: Write>(&self, writer: &mut TableWriter<W>) -> io::Result<()> {
        let mut name_records: Vec<&NameRecord> = self.name_records.iter().collect();
        name_records.sort_by_key(|rec| (rec.platform.to_bytes(), rec.language_id, rec.name_id));

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
        for record in &name_records {
            let [platform_id, encoding_id] = record.platform.to_bytes();
            writer.write_u16::<BigEndian>(platform_id)?;
            writer.write_u16::<BigEndian>(encoding_id)?;
            writer.write_u16::<BigEndian>(record.language_id)?;
            writer.write_u16::<BigEndian>(record.name_id)?;
            let bytes = record.to_bytes();
            let len = bytes.len() as u16;
            writer.write_u16::<BigEndian>(len)?;
            writer.write_u16::<BigEndian>(offset)?;
            offset += len;
            str_buffer.extend(bytes);
        }

        writer.write_all(&str_buffer)?;
        Ok(())
    }
}

struct NameRecord {
    platform: Platform,
    language_id: u16,
    name_id: u16,
    text: String,
}

impl NameRecord {
    fn to_bytes(&self) -> Vec<u8> {
        self.platform.encode(self.language_id, &self.text)
    }
}

// TODO: these constants could use some types..
// and also.. the rest of them..

// Name IDs
pub const COPYRIGHT_NOTICE: u16 = 0;
pub const FONT_FAMILY: u16 = 1;
pub const FONT_SUBFAMILY: u16 = 2;
pub const UNIQUE_SUBFAMILY_ID: u16 = 3;
pub const FULL_FONT_NAME: u16 = 4;
pub const NAME_TABLE_VERSION: u16 = 5;
pub const POSTSCRIPT_NAME: u16 = 6;
pub const DESCRIPTION: u16 = 10;

// Microsoft Languages
pub const ENGLISH_UNITEDSTATES: u16 =  0x0409;
