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

    let correct_lujvo = normalize(&rafsi_list).join("");

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
        if !res.is_empty() && res.last().unwrap().len() != 1 && (lujvo.starts_with('y')
                || lujvo.starts_with("nr") || (lujvo.starts_with('r') && get_cv_info(&lujvo[1..2]) == "C")) {
            res.push(lujvo[0..1].to_string());
            lujvo = lujvo[1..].to_string();
            continue;
        }

        // Drop rafsi from front
        if get_cv_info(&lujvo[0..3]) == "CVV" && ["ai", "ei", "oi", "au"].contains(&&lujvo[1..3]) {
            res.push(lujvo[0..3].to_string());
            lujvo = lujvo[3..].to_string();
            continue;
        }

        if get_cv_info(&lujvo[0..4]) == "CV'V" {
            res.push(lujvo[0..4].to_string());
            lujvo = lujvo[4..].to_string();
            continue;
        }

        if get_cv_info(&lujvo[0..5]) == "CVCCY" || get_cv_info(&lujvo[0..5]) == "CCVCY" {
            res.push(lujvo[0..4].to_string());
            res.push("y".to_string());
            lujvo = lujvo[5..].to_string();
            continue;
        }

        if get_cv_info(&lujvo) == "CVCCV" || get_cv_info(&lujvo) == "CCVCV" {
            res.push(lujvo);
            return Ok(res);
        }

        if get_cv_info(&lujvo[0..3]) == "CVC" || get_cv_info(&lujvo[0..3]) == "CCV" {
            res.push(lujvo[0..3].to_string());
            lujvo = lujvo[3..].to_string();
            continue;
        }

        println!("{:?}, {}", res, lujvo);
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
        assert_eq!(
            jvokaha("bramlatu").unwrap(),
            vec!["bra", "mlatu"]
        );
    }

    #[test]
    fn test_valid_lujvo_toirbroda() {
        assert_eq!(
            jvokaha("toirbroda").unwrap(),
            vec!["toi", "r", "broda"]
        );
    }

    #[test]
    fn test_valid_lujvo_ca_irgau() {
        assert_eq!(
            jvokaha("ca'irgau").unwrap(),
            vec!["ca'i", "r", "gau"]
        );
    }
}

