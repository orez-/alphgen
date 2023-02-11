// https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6name.html
use crate::push_bytes::PushBytes;
use serde::{Serialize, Serializer};

pub(crate) struct Name {
    name_records: Vec<NameRecord>,
}

impl Serialize for Name {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        // format
        let mut buf = vec![0x00, 0x00];
        // number of records
        let len = self.name_records.len() as u16;
        buf.push_be_u16(len);

        // offset to string storage
        // - 12 is sizeof NameRecord
        // - 4 is this byte
        buf.push_be_u16(len * 12 + 4);

        let mut offset = 0;
        let mut str_buffer = Vec::new();
        for record in &self.name_records {
            buf.push_be_u16(record.platform_id);
            buf.push_be_u16(record.encoding_id);
            buf.push_be_u16(record.language_id);
            buf.push_be_u16(record.name_id);
            // TODO: this needs to be UTF-16BE, NOT UTF-8, because we are in hell
            let bytes = record.text.bytes();
            let len = bytes.len() as u16;
            buf.push_be_u16(len);
            buf.push_be_u16(offset);
            offset += len;
            str_buffer.extend(bytes);
        }
        serializer.serialize_bytes(&buf)
    }
}

struct NameRecord {
    platform_id: u16,
    encoding_id: u16,
    language_id: u16,
    name_id: u16,
    text: String,
}
