use super::{scoring::get_lujvo_score, tools::{self, RafsiOptions}};
use once_cell::sync::Lazy;
use serde_json::Value;
use std::collections::HashMap;
use tools::{create_every_possibility, get_candid};

static PERMISSIBILITY_TABLE: Lazy<HashMap<char, HashMap<char, i32>>> = Lazy::new(|| {
    let json: Value = serde_json::from_str(include_str!("permissible.json"))
        .expect("Invalid JSON in permissibility_table.json");
    json.as_object()
        .unwrap()
        .iter()
        .map(|(k, v)| {
            (
                k.chars().next().unwrap(),
                v.as_object()
                    .unwrap()
                    .iter()
                    .map(|(k2, v2)| (k2.chars().next().unwrap(), v2.as_i64().unwrap() as i32))
                    .collect(),
            )
        })
        .collect()
});

#[inline]
fn is_permissible(c1: char, c2: char) -> i32 {
    PERMISSIBILITY_TABLE
        .get(&c1)
        .and_then(|row| row.get(&c2))
        .copied()
        .unwrap_or(0)
}

#[derive(Debug, Clone)]
pub struct LujvoAndScore {
    pub lujvo: String,
    pub score: i32,
}

/// Generate possible lujvo combinations from a list of selrafsi
///
/// # Arguments
/// * `arr` - List of selrafsi (Lojban root words)
/// * `forbid_la_lai_doi` - Whether to forbid certain cmavo in lujvo
/// * `exp_rafsi` - Whether to include experimental rafsi
///
/// # Returns
/// Vector of LujvoAndScore structs sorted by best score first
pub fn jvozba(
    arr: &[String],
    forbid_la_lai_doi: bool,
    forbid_cmevla: bool,
    options: &RafsiOptions,
) -> Vec<LujvoAndScore> {
    let candid_arr: Vec<Vec<String>> = arr
        .iter()
        .enumerate()
        .map(|(i, selrafsi)| get_candid(selrafsi, i == arr.len() - 1, options))
        .collect();

    let mut answers: Vec<LujvoAndScore> = create_every_possibility(candid_arr)
        .into_iter()
        .filter_map(|rafsi_list| {
            normalize(&rafsi_list).ok().map(|result| LujvoAndScore {
                lujvo: result.join(""),
                score: get_lujvo_score(&result),
            })
        })
        .filter(|d| !is_forbidden(d, forbid_la_lai_doi) && !(forbid_cmevla && is_cmevla(&d.lujvo)))
        .collect();

    answers.sort_unstable_by_key(|a| a.score);
    answers
}

#[inline]
fn is_forbidden(d: &LujvoAndScore, forbid_la_lai_doi: bool) -> bool {
    let l = &d.lujvo;
    is_cmevla(l)
        && forbid_la_lai_doi
        && (l.starts_with("lai")
            || l.starts_with("doi")
            || l.contains("lai")
            || l.contains("doi")
            || (l.starts_with("la") && !l.starts_with("lau"))
            || l.split(&['a', 'e', 'i', 'o', 'u', 'y'][..])
                .any(|m| m.starts_with("la") && !m.starts_with("lau")))
}

#[inline]
fn is_cmevla(valsi: &str) -> bool {
    valsi.chars().last().is_some_and(is_c)
}

pub fn normalize(rafsi_list: &[String]) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    if rafsi_list.len() < 2 {
        return Err("You need at least two valsi to make a lujvo".into());
    }

    let mut result: Vec<String> = Vec::with_capacity(rafsi_list.len() * 2 - 1);
    result.push(rafsi_list.last().unwrap().clone());

    for (i, rafsi) in rafsi_list.iter().rev().skip(1).enumerate() {
        let end = rafsi.chars().last().unwrap();
        let init = result[0].chars().next().unwrap();

        if is_4letter(rafsi)
            || (is_c(end) && is_c(init) && is_permissible(end, init) == 0)
            || (end == 'n'
                && ["ts", "tc", "dz", "dj"]
                    .iter()
                    .any(|&s| result[0].starts_with(s)))
        {
            result.insert(0, "y".to_string());
        }

        // Handle CVV case for first rafsi separately
        if i == rafsi_list.len() - 2 && is_cvv(rafsi) {
            let hyphen = if result[0].starts_with('r') { "n" } else { "r" };
            if rafsi_list.len() > 2 || !is_ccv(&result[0]) {
                result.insert(0, hyphen.to_string());
            }
        } else if i == rafsi_list.len() - 2 && is_cvc(rafsi) && is_tosmabru(rafsi, &result) {
            result.insert(0, "y".to_string());
        }

        result.insert(0, rafsi.clone());
    }

    Ok(result)
}

fn is_tosmabru(rafsi: &str, rest: &[String]) -> bool {
    if is_cmevla(rest.last().unwrap()) {
        return false;
    }

    let index = match rest.iter().position(|s| !is_cvc(s)) {
        Some(i) => i,
        None => return false,
    };

    if index < rest.len() {
        let s = &rest[index];
        if s != "y"
            && (get_cv_info(s) != "CVCCV"
                || is_permissible(s.chars().nth(2).unwrap(), s.chars().nth(3).unwrap()) != 2)
        {
            return false;
        }
    }

    let mut tmp1 = rafsi;
    for tmp2 in rest.iter().take(index + 1) {
        if tmp2 == "y" {
            return true;
        }

        let a = tmp1.chars().last().unwrap();
        let b = tmp2.chars().next().unwrap();

        if is_permissible(a, b) != 2 {
            return false;
        }

        tmp1 = tmp2;
    }

    true
}

#[inline]
fn is_cvv(rafsi: &str) -> bool {
    matches!(get_cv_info(rafsi).as_str(), "CVV" | "CV'V")
}

#[inline]
fn is_ccv(rafsi: &str) -> bool {
    get_cv_info(rafsi) == "CCV"
}

#[inline]
fn is_cvc(rafsi: &str) -> bool {
    get_cv_info(rafsi) == "CVC"
}

#[inline]
fn is_4letter(rafsi: &str) -> bool {
    matches!(get_cv_info(rafsi).as_str(), "CVCC" | "CCVC")
}

#[inline]
fn is_c(c: char) -> bool {
    "bcdfgjklmnprstvxz".contains(c)
}

fn get_cv_info(v: &str) -> String {
    v.chars()
        .map(|c| match c {
            'a' | 'e' | 'i' | 'o' | 'u' => "V",
            'b' | 'c' | 'd' | 'f' | 'g' | 'j' | 'k' | 'l' | 'm' | 'n' | 'p' | 'r' | 's' | 't'
            | 'v' | 'x' | 'z' => "C",
            '\'' => "'",
            'y' => "Y",
            _ => "", // Skip unexpected characters
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jvozba_klama_gasnu() {
        let input = vec!["klama".to_string(), "gasnu".to_string()];
        let options = RafsiOptions {
            exp_rafsi: true,
            custom_cmavo: None,
            custom_cmavo_exp: None,
            custom_gismu: None,
            custom_gismu_exp: None,
        };
        let result = jvozba(&input, false, false, &options);

        assert!(
            !result.is_empty(),
            "jvozba should return at least one result"
        );
        assert_eq!(result[0].lujvo, "klagau", "First result should be 'klagau'");
    }

    #[test]
    fn test_jvozba_single_word() {
        let input = vec!["klama".to_string()];
        let options = RafsiOptions {
            exp_rafsi: true,
            custom_cmavo: None,
            custom_cmavo_exp: None,
            custom_gismu: None,
            custom_gismu_exp: None,
        };
        let result = jvozba(&input, false, false, &options);
        assert!(result.is_empty(), "Single word should return empty result");
    }

    #[test]
    fn test_jvozba_empty_input() {
        let input: Vec<String> = vec![];
        let options = RafsiOptions {
            exp_rafsi: true,
            custom_cmavo: None,
            custom_cmavo_exp: None,
            custom_gismu: None,
            custom_gismu_exp: None,
        };
        let result = jvozba(&input, false, false, &options);
        assert!(result.is_empty(), "Empty input should return empty result");
    }

    #[test]
    fn test_jvozba_experimental_rafsi() {
        let input = vec!["klama".to_string(), "gasnu".to_string()];
        let options = RafsiOptions {
            exp_rafsi: true,
            custom_cmavo: None,
            custom_cmavo_exp: None,
            custom_gismu: None,
            custom_gismu_exp: None,
        };
        let result = jvozba(&input, false, false, &options);
        assert!(!result.is_empty(), "Should include experimental rafsi");
    }

    #[test]
    fn test_jvozba_custom_gismu() {
        let mut custom_gismu = HashMap::new();
        custom_gismu.insert("klama".into(), vec!["qla".into()]);
        let mut custom_gismu_exp = HashMap::new();
        custom_gismu_exp.insert("gasnu".into(), vec!["gasn".into()]);
        
        let input = vec!["klama".to_string(), "gasnu".to_string()];
        let options = RafsiOptions {
            exp_rafsi: true,
            custom_cmavo: None,
            custom_cmavo_exp: None,
            custom_gismu: Some(&custom_gismu),
            custom_gismu_exp: Some(&custom_gismu_exp),
        };
        
        let result = jvozba(&input, false, false, &options);
        assert!(!result.is_empty(), "Should use custom gismu rafsi");
        assert!(result.iter().any(|r| r.lujvo == "qlagasnu"), "Expected custom rafsi combination");
    }

    #[test]
    fn test_is_tosmabru() {
        // Test a valid tosmabru case
        let rafsi = "tos";
        let rest = vec!["mabru".to_string()];
        assert!(
            is_tosmabru(rafsi, &rest),
            "'tosmabru' should be a valid tosmabru"
        );

        // Test invalid case
        let rafsi = "bad";
        let rest = vec!["example".to_string()];
        assert!(
            !is_tosmabru(rafsi, &rest),
            "Invalid tosmabru case should return false"
        );
    }

    #[test]
    fn test_normalize() {
        let input = vec!["slak".to_string(), "gau".to_string()];
        let result = normalize(&input).unwrap();
        assert_eq!(
            result,
            vec!["slak", "y", "gau"],
            "Normalization should insert y-hyphen"
        );
    }

    #[test]
    fn test_normalize_error() {
        let input = vec!["klama".to_string()];
        let result = normalize(&input);
        assert!(result.is_err(), "Normalizing single word should error");
    }

    #[test]
    fn test_is_cmevla() {
        assert!(is_cmevla("klaman"), "Should recognize cmevla");
        assert!(!is_cmevla("klama"), "Should recognize non-cmevla");
    }
}
