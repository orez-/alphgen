pub(crate) enum Platform {
    Unicode(UnicodeEncoding),
    Microsoft(MicrosoftEncoding),
}

impl Platform {
    pub(crate) fn unicode_2_0() -> Self {
        Platform::Unicode(UnicodeEncoding::Unicode2_0)
    }

    pub(crate) fn microsoft_bmp() -> Self {
        Platform::Microsoft(MicrosoftEncoding::UnicodeBMP)
    }

    pub(crate) fn to_bytes(&self) -> [u16; 2] {
        match self {
            Platform::Unicode(UnicodeEncoding::Unicode2_0) => [0, 3],
            Platform::Microsoft(MicrosoftEncoding::UnicodeBMP) => [3, 1],
        }
    }
}

pub(crate) enum UnicodeEncoding {
    Unicode2_0,
}

pub(crate) enum MicrosoftEncoding {
    UnicodeBMP,
}
