use crate::error::{Result, VlazbaError};
use crate::morphology::cv::{CvInfo, cv_shape};
use crate::morphology::compound::normalize;

/// Split a lujvo into its constituent rafsi (including single-letter hyphens).
///
/// Forms that differ from the score-normalized spelling only by optional
/// hyphens (e.g. `rivyzu'e` vs canonical `rivzu'e`) are accepted: the
/// canonical form is re-decomposed and returned.
pub fn jvokaha(lujvo: &str) -> Result<Vec<String>> {
    let arr = decompose_into_rafsi(lujvo)?;
    let rafsi_list: Vec<String> = arr.iter().filter(|a| a.len() != 1).cloned().collect();

    let correct_lujvo = normalize(&rafsi_list)?.join("");
    if lujvo == correct_lujvo {
        Ok(arr)
    } else {
        decompose_into_rafsi(&correct_lujvo)
    }
}

/// Internal decomposition using a byte/char index cursor (ASCII Lojban).
fn decompose_into_rafsi(lujvo: &str) -> Result<Vec<String>> {
    let original_lujvo = lujvo;
    // Classical Lojban orthography is ASCII; reject early so byte indexing is safe.
    if !lujvo.is_ascii() {
        return Err(VlazbaError::Decompose(original_lujvo.to_string()));
    }
    let mut res: Vec<String> = Vec::new();
    let mut i = 0;
    let len = lujvo.len();

    while i < len {
        let rest = &lujvo[i..];

        if !res.is_empty() && res.last().unwrap().len() != 1 {
            let first_char = rest.chars().next().ok_or(VlazbaError::UnexpectedEof)?;
            let second_char = rest.chars().nth(1);

            if first_char == 'y'
                || (first_char == 'n' && second_char == Some('r'))
                || (first_char == 'r'
                    && second_char.is_some_and(|c| cv_shape(&c.to_string()) == CvInfo::C))
            {
                res.push(first_char.to_string());
                i += first_char.len_utf8();
                continue;
            }
        }

        let remaining = len - i;

        if remaining >= 3 {
            let first_three = &lujvo[i..i + 3];
            if cv_shape(first_three) == CvInfo::CVV {
                let middle_two = &lujvo[i + 1..i + 3];
                if matches!(middle_two, "ai" | "ei" | "oi" | "au") {
                    res.push(first_three.to_string());
                    i += 3;
                    continue;
                }
            }
        }

        if remaining >= 4 {
            let first_four = &lujvo[i..i + 4];
            if cv_shape(first_four) == CvInfo::CVApostropheV {
                res.push(first_four.to_string());
                i += 4;
                continue;
            }
        }

        if remaining >= 5 {
            let first_five = &lujvo[i..i + 5];
            match cv_shape(first_five) {
                CvInfo::CVCCY | CvInfo::CCVCY => {
                    res.push(lujvo[i..i + 4].to_string());
                    res.push("y".to_string());
                    i += 5;
                    continue;
                }
                _ => {}
            }
        }

        match cv_shape(rest) {
            CvInfo::CVCCV | CvInfo::CCVCV => {
                res.push(rest.to_string());
                return Ok(res);
            }
            _ => {}
        }

        if remaining >= 3 {
            let first_three = &lujvo[i..i + 3];
            match cv_shape(first_three) {
                CvInfo::CVC | CvInfo::CCV => {
                    res.push(first_three.to_string());
                    i += 3;
                    continue;
                }
                _ => {}
            }
        }

        return Err(VlazbaError::Decompose(original_lujvo.to_string()));
    }

    if res.is_empty() {
        return Err(VlazbaError::Decompose(original_lujvo.to_string()));
    }

    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_lujvo_bramlatu() {
        assert_eq!(jvokaha("bramlatu").unwrap(), vec!["bra", "mlatu"]);
    }

    #[test]
    fn test_valid_lujvo_toirbroda() {
        assert_eq!(jvokaha("toirbroda").unwrap(), vec!["toi", "r", "broda"]);
    }

    #[test]
    fn test_valid_lujvo_ca_irgau() {
        assert_eq!(jvokaha("ca'irgau").unwrap(), vec!["ca'i", "r", "gau"]);
    }

    #[test]
    fn test_valid_lujvo_with_y_hyphen() {
        assert_eq!(
            jvokaha("klamyseltru").unwrap(),
            vec!["klam", "y", "sel", "tru"]
        );
    }

    #[test]
    fn test_valid_lujvo_with_nr_hyphen() {
        assert!(jvokaha("toinrbroda").is_err());
    }

    #[test]
    fn test_invalid_klasr() {
        assert!(jvokaha("klasr").is_err());
    }

    #[test]
    fn test_invalid_empty() {
        assert!(jvokaha("").is_err());
    }

    #[test]
    fn test_invalid_cyrillic() {
        assert!(jvokaha("щя").is_err());
    }

    #[test]
    fn test_invalid_multibyte() {
        assert!(jvokaha("café").is_err());
        assert!(jvokaha("日本語").is_err());
        assert!(jvokaha("😀").is_err());
    }

    #[test]
    fn test_invalid_short_lujvo() {
        assert!(jvokaha("la").is_err());
    }

    #[test]
    fn test_invalid_rafsi_sequence() {
        assert!(jvokaha("klamrseltru").is_err());
    }

    #[test]
    fn test_jvokaha2_valid() {
        let result = decompose_into_rafsi("bramlatu").unwrap();
        assert_eq!(result, vec!["bra", "mlatu"]);
    }

    #[test]
    fn test_jvokaha2_invalid() {
        assert!(decompose_into_rafsi("invalid").is_err());
    }

    #[test]
    fn test_jvokaha_cvc_y_cvv() {
        assert_eq!(jvokaha("zukyde'a").unwrap(), vec!["zuk", "y", "de'a"]);
    }

    #[test]
    fn test_jvokaha_optional_extra_y() {
        assert_eq!(jvokaha("rivzu'e").unwrap(), vec!["riv", "zu'e"]);
        assert_eq!(jvokaha("rivyzu'e").unwrap(), vec!["riv", "zu'e"]);
    }
}
