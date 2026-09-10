//! Consonant/vowel shape classification without heap allocation.

/// CV-shape of a rafsi or hyphen fragment (ASCII Lojban orthography).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CvInfo {
    C,
    Y,
    CVC,
    CCV,
    CVV,
    /// `CV'V`
    CVApostropheV,
    CVCC,
    CCVC,
    CVCCV,
    CCVCV,
    CVCCY,
    CCVCY,
    Other,
}

impl CvInfo {
    #[inline]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::C => "C",
            Self::Y => "Y",
            Self::CVC => "CVC",
            Self::CCV => "CCV",
            Self::CVV => "CVV",
            Self::CVApostropheV => "CV'V",
            Self::CVCC => "CVCC",
            Self::CCVC => "CCVC",
            Self::CVCCV => "CVCCV",
            Self::CCVCV => "CCVCV",
            Self::CVCCY => "CVCCY",
            Self::CCVCY => "CCVCY",
            Self::Other => "",
        }
    }

    #[inline]
    pub const fn is_cvv(self) -> bool {
        matches!(self, Self::CVV | Self::CVApostropheV)
    }

    #[inline]
    pub const fn is_ccv(self) -> bool {
        matches!(self, Self::CCV)
    }

    #[inline]
    pub const fn is_cvc(self) -> bool {
        matches!(self, Self::CVC)
    }

    #[inline]
    pub const fn is_4letter(self) -> bool {
        matches!(self, Self::CVCC | Self::CCVC)
    }
}

impl std::fmt::Display for CvInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[inline]
const fn class_byte(b: u8) -> u8 {
    match b {
        b'a' | b'e' | b'i' | b'o' | b'u' => b'V',
        b'b' | b'c' | b'd' | b'f' | b'g' | b'j' | b'k' | b'l' | b'm' | b'n' | b'p' | b'r' | b's'
        | b't' | b'v' | b'x' | b'z' => b'C',
        b'\'' => b'\'',
        b'y' => b'Y',
        _ => 0,
    }
}

/// Classify a Lojban fragment's CV shape. Non-ASCII / unexpected bytes yield [`CvInfo::Other`].
#[inline]
pub fn cv_shape(v: &str) -> CvInfo {
    let bytes = v.as_bytes();
    if bytes.is_empty() || bytes.len() > 5 {
        return CvInfo::Other;
    }

    let mut buf = [0u8; 5];
    for (i, &b) in bytes.iter().enumerate() {
        let c = class_byte(b);
        if c == 0 {
            return CvInfo::Other;
        }
        buf[i] = c;
    }

    match bytes.len() {
        1 => match buf[0] {
            b'C' => CvInfo::C,
            b'Y' => CvInfo::Y,
            _ => CvInfo::Other,
        },
        3 => match &buf[..3] {
            b"CVC" => CvInfo::CVC,
            b"CCV" => CvInfo::CCV,
            b"CVV" => CvInfo::CVV,
            _ => CvInfo::Other,
        },
        4 => match &buf[..4] {
            b"CV'V" => CvInfo::CVApostropheV,
            b"CVCC" => CvInfo::CVCC,
            b"CCVC" => CvInfo::CCVC,
            _ => CvInfo::Other,
        },
        5 => match &buf[..5] {
            b"CVCCV" => CvInfo::CVCCV,
            b"CCVCV" => CvInfo::CCVCV,
            b"CVCCY" => CvInfo::CVCCY,
            b"CCVCY" => CvInfo::CCVCY,
            _ => CvInfo::Other,
        },
        _ => CvInfo::Other,
    }
}

/// Legacy string form of [`cv_shape`] (allocates). Prefer [`cv_shape`].
pub fn cv_shape_string(v: &str) -> String {
    cv_shape(v).as_str().to_owned()
}

#[inline]
pub fn is_consonant(c: char) -> bool {
    matches!(
        c,
        'b' | 'c' | 'd' | 'f' | 'g' | 'j' | 'k' | 'l' | 'm' | 'n' | 'p' | 'r' | 's' | 't' | 'v'
            | 'x' | 'z'
    )
}

/// Compatibility alias.
#[inline]
pub fn classify_cv(v: &str) -> CvInfo {
    cv_shape(v)
}

/// Compatibility alias.
#[inline]
pub fn get_cv_info(v: &str) -> String {
    cv_shape_string(v)
}

/// Compatibility alias.
#[inline]
pub fn is_c(c: char) -> bool {
    is_consonant(c)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_common_shapes() {
        assert_eq!(cv_shape("y"), CvInfo::Y);
        assert_eq!(cv_shape("r"), CvInfo::C);
        assert_eq!(cv_shape("kla"), CvInfo::CCV);
        assert_eq!(cv_shape("gau"), CvInfo::CVV);
        assert_eq!(cv_shape("ca'i"), CvInfo::CVApostropheV);
        assert_eq!(cv_shape("klam"), CvInfo::CCVC);
        assert_eq!(cv_shape("mlatu"), CvInfo::CCVCV);
        assert_eq!(cv_shape("broda"), CvInfo::CCVCV);
        assert_eq!(cv_shape("café"), CvInfo::Other);
    }
}
