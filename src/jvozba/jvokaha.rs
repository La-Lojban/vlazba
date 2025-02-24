use crate::jvozba::scoring::get_cv_info;
use std::error::Error;
use std::fmt;

use super::jvozbanarge::normalize;

#[derive(Debug)]
struct LujvoError {
    message: String,
}

impl fmt::Display for LujvoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for LujvoError {}

/// Split a lujvo into its constituent rafsi
///
/// # Arguments
/// * `lujvo` - The compound word to analyze
///
/// # Returns
/// Result with vector of rafsi or error message
pub fn jvokaha(lujvo: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let arr = jvokaha2(lujvo)?;
    let rafsi_list: Vec<String> = arr.iter().filter(|a| a.len() != 1).cloned().collect();

    let correct_lujvo = normalize(&rafsi_list)?.join("");
    if lujvo == correct_lujvo {
        Ok(arr)
    } else {
        Err(Box::new(LujvoError {
            message: format!(
                "malformed lujvo {{{}}}; it should be {{{}}}",
                lujvo, correct_lujvo
            ),
        }))
    }
}

fn jvokaha2(lujvo: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let original_lujvo = lujvo.to_string();
    let mut res: Vec<String> = Vec::new();
    let mut lujvo = lujvo.to_string();

    while !lujvo.is_empty() {
        // Remove hyphen
        if !res.is_empty()
            && res.last().unwrap().len() != 1
        {
            let first_char = lujvo.chars().next().ok_or_else(|| LujvoError {
                message: "Unexpected end of input".to_string(),
            })?;
            
            let second_char = lujvo.chars().nth(1);
            
            if first_char == 'y'
                || (first_char == 'n' && second_char == Some('r'))
                || (first_char == 'r' && second_char.is_some_and(|c| get_cv_info(&c.to_string()) == "C"))
            {
                res.push(first_char.to_string());
                lujvo = lujvo.chars().skip(1).collect();
                continue;
            }
        }

        // Drop rafsi from front
        if lujvo.chars().count() >= 3 {
            let first_three: String = lujvo.chars().take(3).collect();
            if get_cv_info(&first_three) == "CVV" {
                let middle_two: String = lujvo.chars().skip(1).take(2).collect();
                if ["ai", "ei", "oi", "au"].contains(&middle_two.as_str()) {
                    res.push(first_three);
                    lujvo = lujvo.chars().skip(3).collect();
                    continue;
                }
            }
        }

        if lujvo.chars().count() >= 4 {
            let first_four: String = lujvo.chars().take(4).collect();
            if get_cv_info(&first_four) == "CV'V" {
                res.push(first_four);
                lujvo = lujvo.chars().skip(4).collect();
                continue;
            }
        }

        if lujvo.chars().count() >= 5 {
            let first_five: String = lujvo.chars().take(5).collect();
            if get_cv_info(&first_five) == "CVCCY" || get_cv_info(&first_five) == "CCVCY" {
                let first_four: String = lujvo.chars().take(4).collect();
                res.push(first_four);
                res.push("y".to_string());
                lujvo = lujvo.chars().skip(5).collect();
                continue;
            }
        }

        let cv_info = get_cv_info(&lujvo);
        if cv_info == "CVCCV" || cv_info == "CCVCV" {
            res.push(lujvo);
            return Ok(res);
        }

        if lujvo.chars().count() >= 3 {
            let first_three: String = lujvo.chars().take(3).collect();
            let cv_info = get_cv_info(&first_three);
            if cv_info == "CVC" || cv_info == "CCV" {
                res.push(first_three);
                lujvo = lujvo.chars().skip(3).collect();
                continue;
            }
        }

        return Err(Box::new(LujvoError {
            message: format!("Failed to decompose {{{}}}", original_lujvo),
        }));
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
        assert_eq!(jvokaha("klamyseltru").unwrap(), vec!["klam", "y", "sel", "tru"]);
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
        // Test with a multibyte character sequence
        assert!(jvokaha("café").is_err());
        // Test with a Japanese character
        assert!(jvokaha("日本語").is_err());
        // Test with emoji
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
        let result = jvokaha2("bramlatu").unwrap();
        assert_eq!(result, vec!["bra", "mlatu"]);
    }

    #[test]
    fn test_jvokaha2_invalid() {
        assert!(jvokaha2("invalid").is_err());
    }
}
