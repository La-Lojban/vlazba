use super::cv::{CvInfo, cv_shape};

pub use super::cv::{cv_shape_string, is_consonant};

pub fn lujvo_score(rafsi_ynr_sequence: &[String]) -> i32 {
    let lujvo = rafsi_ynr_sequence.join("");
    let l = lujvo.len() as i32;
    let a = lujvo.bytes().filter(|&b| b == b'\'').count() as i32;
    let mut h = 0;
    let mut r = 0;

    for rafsi in rafsi_ynr_sequence {
        match cv_shape(rafsi) {
            CvInfo::C | CvInfo::Y => h += 1,
            CvInfo::CVCCV => r += 1,
            CvInfo::CVCC => r += 2,
            CvInfo::CCVCV => r += 3,
            CvInfo::CCVC => r += 4,
            CvInfo::CVC => r += 5,
            CvInfo::CVApostropheV => r += 6,
            CvInfo::CCV => r += 7,
            CvInfo::CVV => r += 8,
            _ => {}
        }
    }

    let v = lujvo
        .bytes()
        .filter(|&c| matches!(c, b'a' | b'e' | b'i' | b'o' | b'u'))
        .count() as i32;

    (1000 * l) - (500 * a) + (100 * h) - (10 * r) - v
}

/// Compatibility alias for [`lujvo_score`].
pub fn get_lujvo_score(rafsi_ynr_sequence: &[String]) -> i32 {
    lujvo_score(rafsi_ynr_sequence)
}

pub use super::cv::{get_cv_info, is_c};
