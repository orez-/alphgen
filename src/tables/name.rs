// https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6name.html
use crate::{FontTable, TableWriter};
use crate::platform::Platform;
use std::io::{self, Write};
use byteorder::{BigEndian, WriteBytesExt};

pub(crate) struct Name {
    name_records: Vec<NameRecord>,
}

impl Name {
    // XXX: 👀
    // ...are you allowed to just not have any names??
    pub(crate) fn new() -> Self {
        Name { name_records: Vec::new() }
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
        // - 4 is this byte
        writer.write_u16::<BigEndian>(len * 12 + 4)?;

        let mut offset = 0;
        let mut str_buffer = Vec::new();
        for record in &self.name_records {
            let [platform_id, encoding_id] = record.platform.to_bytes();
            writer.write_u16::<BigEndian>(platform_id)?;
            writer.write_u16::<BigEndian>(encoding_id)?;
            writer.write_u16::<BigEndian>(record.language_id)?;
            writer.write_u16::<BigEndian>(record.name_id)?;
            // TODO: this needs to be UTF-16BE, NOT UTF-8, because we are in hell
            let bytes = record.text.bytes();
            let len = bytes.len() as u16;
            writer.write_u16::<BigEndian>(len)?;
            writer.write_u16::<BigEndian>(offset)?;
            offset += len;
            str_buffer.extend(bytes);
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
