pub trait PushBytes {
    fn push_be_u16(&mut self, bytes: u16);
    fn push_le_u16(&mut self, bytes: u16);
    fn push_be_u32(&mut self, bytes: u32);
    fn push_le_u32(&mut self, bytes: u32);
    fn push_be_i16(&mut self, bytes: i16);
    fn push_le_i16(&mut self, bytes: i16);
    fn push_be_i64(&mut self, bytes: i64);
    fn push_le_i64(&mut self, bytes: i64);
}

impl PushBytes for Vec<u8> {
    fn push_be_u16(&mut self, bytes: u16) {
        self.extend(bytes.to_be_bytes());
    }

    fn push_le_u16(&mut self, bytes: u16) {
        self.extend(bytes.to_le_bytes());
    }

    fn push_be_u32(&mut self, bytes: u32) {
        self.extend(bytes.to_be_bytes());
    }

    fn push_le_u32(&mut self, bytes: u32) {
        self.extend(bytes.to_le_bytes());
    }

    fn push_be_i16(&mut self, bytes: i16) {
        self.extend(bytes.to_be_bytes());
    }

    fn push_le_i16(&mut self, bytes: i16) {
        self.extend(bytes.to_le_bytes());
    }

    fn push_be_i64(&mut self, bytes: i64) {
        self.extend(bytes.to_be_bytes());
    }

    fn push_le_i64(&mut self, bytes: i64) {
        self.extend(bytes.to_le_bytes());
    }
}
