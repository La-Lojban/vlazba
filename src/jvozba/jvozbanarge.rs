use super::{scoring::get_lujvo_score, tools};
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
pub fn jvozba(arr: &[String], forbid_la_lai_doi: bool, exp_rafsi: bool) -> Vec<LujvoAndScore> {
    let candid_arr: Vec<Vec<String>> = arr
        .iter()
        .enumerate()
        .map(|(i, selrafsi)| get_candid(selrafsi, i == arr.len() - 1, exp_rafsi))
        .collect();

    let mut answers: Vec<LujvoAndScore> = create_every_possibility(candid_arr)
        .into_iter()
        .filter_map(|rafsi_list| {
            normalize(&rafsi_list).ok().map(|result| LujvoAndScore {
                lujvo: result.join(""),
                score: get_lujvo_score(&result),
            })
        })
        .filter(|d| !is_forbidden(d, forbid_la_lai_doi))
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
    valsi.chars().last().is_some_and(|c| !"aeiouy'".contains(c))
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

    let index = rest
        .iter()
        .position(|s| !is_cvc(s))
        .unwrap_or_else(|| panic!("Cannot happen"));

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
            'a' | 'e' | 'i' | 'o' | 'u' => 'V',
            'b' | 'c' | 'd' | 'f' | 'g' | 'j' | 'k' | 'l' | 'm' | 'n' | 'p' | 'r' | 's' | 't'
            | 'v' | 'x' | 'z' => 'C',
            '\'' => '\'',
            'y' => 'Y',
            _ => panic!("Unexpected character: {}", c),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jvozba_klama_gasnu() {
        let input = vec!["klama".to_string(), "gasnu".to_string()];
        let result = jvozba(&input, false, false);

        assert!(
            !result.is_empty(),
            "jvozba should return at least one result"
        );
        assert_eq!(result[0].lujvo, "klagau", "First result should be 'klagau'");
    }
}
