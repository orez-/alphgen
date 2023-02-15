pub(crate) enum Platform {
    Unicode(UnicodeEncoding),
}

impl Platform {
    pub(crate) fn unicode_2_0() -> Self {
        Platform::Unicode(UnicodeEncoding::Unicode2_0)
    }

    pub(crate) fn to_bytes(&self) -> [u16; 2] {
        match self {
            Platform::Unicode(UnicodeEncoding::Unicode2_0) => [0, 3],
        }
    }
}

pub(crate) enum UnicodeEncoding {
    Unicode2_0,
}
