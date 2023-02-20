pub(crate) enum Platform {
    Unicode(UnicodeEncoding),
    Macintosh(MacintoshEncoding),
    Microsoft(MicrosoftEncoding),
}

impl Platform {
    pub(crate) fn unicode_2_0() -> Self {
        Platform::Unicode(UnicodeEncoding::Unicode2_0)
    }

    pub(crate) fn macintosh_roman() -> Self {
        Platform::Macintosh(MacintoshEncoding::Roman)
    }

    pub(crate) fn microsoft_bmp() -> Self {
        Platform::Microsoft(MicrosoftEncoding::UnicodeBMP)
    }

    pub(crate) fn to_bytes(&self) -> [u16; 2] {
        match self {
            Platform::Unicode(UnicodeEncoding::Unicode2_0) => [0, 3],
            Platform::Macintosh(MacintoshEncoding::Roman) => [1, 0],
            Platform::Microsoft(MicrosoftEncoding::UnicodeBMP) => [3, 1],
        }
    }

    pub(crate) fn encode(&self, language_id: u16, text: &str) -> Vec<u8> {
        match (self, language_id) {
            (Platform::Unicode(UnicodeEncoding::Unicode2_0), _) |
            (Platform::Microsoft(MicrosoftEncoding::UnicodeBMP), _) => {
                text.encode_utf16()
                    .flat_map(|pair| pair.to_be_bytes())
                    .collect()
            }
            (Platform::Macintosh(MacintoshEncoding::Roman), _) => {
                text.as_bytes().to_vec()  // TODO: this is wrong
            }
        }
    }
}

pub(crate) enum UnicodeEncoding {
    Unicode2_0,
}

pub(crate) enum MacintoshEncoding {
    Roman,
}

pub(crate) enum MicrosoftEncoding {
    UnicodeBMP,
}
