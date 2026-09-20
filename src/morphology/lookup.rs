use std::collections::HashMap;

use super::rafsi_tables::{
    get_cmavo_rafsi_list, get_cmavo_rafsi_list_exp, get_gismu_rafsi_list, get_gismu_rafsi_list_exp,
    reverse_cmavo, reverse_cmavo_exp, reverse_gismu, reverse_gismu_exp,
};
use super::{compound, decompose};
use crate::error::{Result, VlazbaError};

#[derive(Clone)]
pub struct RafsiOptions<'a> {
    pub exp_rafsi: bool,
    pub custom_cmavo: Option<&'a HashMap<String, Vec<String>>>,
    pub custom_cmavo_exp: Option<&'a HashMap<String, Vec<String>>>,
    pub custom_gismu: Option<&'a HashMap<String, Vec<String>>>,
    pub custom_gismu_exp: Option<&'a HashMap<String, Vec<String>>>,
}

/// Return the implicit four-letter rafsi of a five-letter vowel-final gismu.
///
/// The `brod` stem is excluded because the broda series does not receive this
/// implicit rafsi. Callers are responsible for supplying a word known to be a
/// gismu; this helper deliberately does not perform full morphology parsing.
pub fn implicit_four_letter_gismu_rafsi(gismu: &str) -> Option<String> {
    let mut chars = gismu.chars();
    let stem: String = chars.by_ref().take(4).collect();
    let final_vowel = chars.next()?;
    if chars.next().is_some()
        || !matches!(final_vowel, 'a' | 'e' | 'i' | 'o' | 'u')
        || stem == "brod"
    {
        return None;
    }
    Some(stem)
}

/// Cartesian product of candidate lists (clone-based; no JSON roundtrip).
pub fn cartesian_product<T: Clone>(aa: Vec<Vec<T>>) -> Vec<Vec<T>> {
    if aa.is_empty() {
        return vec![vec![]];
    }
    aa.into_iter().fold(vec![vec![]], |acc, list| {
        let mut result = Vec::with_capacity(acc.len().saturating_mul(list.len().max(1)));
        for prefix in &acc {
            for item in &list {
                let mut row = prefix.clone();
                row.push(item.clone());
                result.push(row);
            }
        }
        result
    })
}

pub fn gismu_rafsi_list(
    a: &str,
    exp_rafsi: bool,
    custom_gismu: Option<&HashMap<String, Vec<String>>>,
    custom_gismu_exp: Option<&HashMap<String, Vec<String>>>,
) -> Option<Vec<String>> {
    if let Some(custom_gismu) = custom_gismu {
        if let Some(rafsi) = custom_gismu.get(a) {
            return Some(rafsi.clone());
        }
    }
    if let Some(rafsi) = get_gismu_rafsi_list().get(a) {
        if !rafsi.is_empty() {
            return Some(rafsi.clone());
        }
    }

    if exp_rafsi {
        if let Some(custom_gismu_exp) = custom_gismu_exp {
            if let Some(rafsi) = custom_gismu_exp.get(a) {
                return Some(rafsi.clone());
            }
        }
        if let Some(rafsi) = get_gismu_rafsi_list_exp().get(a) {
            if !rafsi.is_empty() {
                return Some(rafsi.clone());
            }
        }
    }
    Some(Vec::<String>::new())
}

pub fn cmavo_rafsi_list(
    a: &str,
    exp_rafsi: bool,
    custom_cmavo: Option<&HashMap<String, Vec<String>>>,
    custom_cmavo_exp: Option<&HashMap<String, Vec<String>>>,
) -> Option<Vec<String>> {
    if let Some(custom_cmavo) = custom_cmavo {
        if let Some(rafsi) = custom_cmavo.get(a) {
            return Some(rafsi.clone());
        }
    }
    if let Some(rafsi) = get_cmavo_rafsi_list().get(a) {
        if !rafsi.is_empty() {
            return Some(rafsi.clone());
        }
    }

    if exp_rafsi {
        if let Some(custom_cmavo_exp) = custom_cmavo_exp {
            if let Some(rafsi) = custom_cmavo_exp.get(a) {
                return Some(rafsi.clone());
            }
        }
        if let Some(rafsi) = get_cmavo_rafsi_list_exp().get(a) {
            if !rafsi.is_empty() {
                return Some(rafsi.clone());
            }
        }
    }
    None
}

pub fn rafsi_candidates(selrafsi: &str, is_last: bool, options: &RafsiOptions) -> Vec<String> {
    if let Some(a) = cmavo_rafsi_list(
        selrafsi,
        options.exp_rafsi,
        options.custom_cmavo,
        options.custom_cmavo_exp,
    ) {
        return a;
    }
    if let Some(b) = gismu_rafsi_list(
        selrafsi,
        options.exp_rafsi,
        options.custom_gismu,
        options.custom_gismu_exp,
    ) {
        let gismu = selrafsi;
        let mut candid = b;

        if is_last {
            candid.push(gismu.to_string());
        }

        if let Some(implicit) = implicit_four_letter_gismu_rafsi(gismu) {
            candid.push(implicit);
        }
        candid
    } else {
        Vec::new()
    }
}

/// Reconstruct a lujvo from its components.
///
/// When custom rafsi maps are incomplete, missing keys fall through to the
/// built-in lists (per-key override, not wholesale replace). A full builtin
/// retry is kept for other reconstruct failures.
pub fn reconstruct_lujvo(
    lujvo: &str,
    forbid_cmevla: bool,
    options: &RafsiOptions,
) -> Result<String> {
    let classical = match reconstruct_lujvo_with(lujvo, forbid_cmevla, options) {
        Ok(s) => Ok(s),
        Err(e) => {
            let has_custom = options.custom_cmavo.is_some()
                || options.custom_cmavo_exp.is_some()
                || options.custom_gismu.is_some()
                || options.custom_gismu_exp.is_some();
            if has_custom {
                let builtin = RafsiOptions {
                    exp_rafsi: options.exp_rafsi,
                    custom_cmavo: None,
                    custom_cmavo_exp: None,
                    custom_gismu: None,
                    custom_gismu_exp: None,
                };
                reconstruct_lujvo_with(lujvo, forbid_cmevla, &builtin)
            } else {
                Err(e)
            }
        }
    };
    #[cfg(feature = "camxes")]
    if classical.is_err() {
        if let Some(rebuilt) = super::camxes_segments::reconstruct_with_inferred_sources(lujvo, options) {
            return Ok(rebuilt);
        }
    }
    classical
}

fn reconstruct_lujvo_with(
    lujvo: &str,
    forbid_cmevla: bool,
    options: &RafsiOptions,
) -> Result<String> {
    let rafsi_list = decompose::jvokaha(lujvo)?;

    let mut selrafsi_list = Vec::new();
    for rafsi in &rafsi_list {
        if rafsi == "y" || rafsi == "r" || rafsi == "n" {
            continue;
        }
        let Some(selrafsi) = resolve_selrafsi(rafsi, options) else {
            return Err(VlazbaError::UnresolvedRafsi {
                rafsi: rafsi.clone(),
                lujvo: lujvo.to_string(),
            });
        };
        selrafsi_list.push(selrafsi);
    }

    if selrafsi_list.len() < 2 {
        return Err(VlazbaError::TooFewSelrafsi);
    }

    let rebuilt = compound::jvozba(&selrafsi_list, false, forbid_cmevla, true, options)
        .first()
        .ok_or(VlazbaError::RebuildFailed)?
        .lujvo
        .clone();

    Ok(rebuilt)
}

/// Score-optimal spelling analysis for a lujvo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LujvoSpellingAnalysis {
    /// Best-scoring form from the same selrafsi (`reconstruct_lujvo`).
    pub canonical: String,
    /// True when `canonical` equals the input spelling.
    pub is_score_optimal: bool,
}

/// Compare a lujvo spelling to its score-optimal form.
///
/// Returns `None` if the string cannot be reconstructed.
pub fn analyze_lujvo_spelling(
    word: &str,
    options: &RafsiOptions,
) -> Option<LujvoSpellingAnalysis> {
    let canonical = reconstruct_lujvo(word, true, options).ok()?;
    Some(LujvoSpellingAnalysis {
        is_score_optimal: canonical == word,
        canonical,
    })
}

fn find_in_custom(map: &HashMap<String, Vec<String>>, needle: &str) -> Option<String> {
    map.iter()
        .find(|(_, list)| list.iter().any(|r| r == needle))
        .map(|(k, _)| k.clone())
}

pub fn resolve_selrafsi(rafsi: &str, options: &RafsiOptions) -> Option<String> {
    if gismu_key_exists(rafsi, options) {
        return Some(rafsi.to_owned());
    }

    if rafsi != "brod" && rafsi.len() == 4 && !rafsi.contains('\'') {
        for vowel in "aeiou".chars() {
            let gismu_candid = format!("{}{}", rafsi, vowel);
            if gismu_key_exists(&gismu_candid, options) {
                return Some(gismu_candid);
            }
        }
    }

    if let Some(m) = options.custom_gismu {
        if let Some(found) = find_in_custom(m, rafsi) {
            return Some(found);
        }
    }
    if let Some(found) = reverse_gismu(rafsi) {
        return Some(found.to_owned());
    }

    if let Some(m) = options.custom_cmavo {
        if let Some(found) = find_in_custom(m, rafsi) {
            return Some(found);
        }
    }
    if let Some(found) = reverse_cmavo(rafsi) {
        return Some(found.to_owned());
    }

    if options.exp_rafsi {
        if let Some(m) = options.custom_gismu_exp {
            if let Some(found) = find_in_custom(m, rafsi) {
                return Some(found);
            }
        }
        if let Some(found) = reverse_gismu_exp(rafsi) {
            return Some(found.to_owned());
        }

        if let Some(m) = options.custom_cmavo_exp {
            if let Some(found) = find_in_custom(m, rafsi) {
                return Some(found);
            }
        }
        if let Some(found) = reverse_cmavo_exp(rafsi) {
            return Some(found.to_owned());
        }
    }

    None
}

fn gismu_key_exists(candid: &str, options: &RafsiOptions) -> bool {
    if let Some(m) = options.custom_gismu {
        if m.contains_key(candid) {
            return true;
        }
    }
    if get_gismu_rafsi_list().contains_key(candid) {
        return true;
    }
    if options.exp_rafsi {
        if let Some(m) = options.custom_gismu_exp {
            if m.contains_key(candid) {
                return true;
            }
        }
        if get_gismu_rafsi_list_exp().contains_key(candid) {
            return true;
        }
    }
    false
}



/// Compatibility alias.
pub fn create_every_possibility<T: Clone>(aa: Vec<Vec<T>>) -> Vec<Vec<T>> {
    cartesian_product(aa)
}

/// Compatibility alias.
pub fn get_candid(selrafsi: &str, is_last: bool, options: &RafsiOptions) -> Vec<String> {
    rafsi_candidates(selrafsi, is_last, options)
}

/// Compatibility alias (legacy name with trailing `2`).
pub fn search_selrafsi_from_rafsi2(rafsi: &str, options: &RafsiOptions) -> Option<String> {
    resolve_selrafsi(rafsi, options)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn implicit_four_letter_rafsi_obeys_gismu_rule() {
        assert_eq!(implicit_four_letter_gismu_rafsi("blanu").as_deref(), Some("blan"));
        assert_eq!(implicit_four_letter_gismu_rafsi("mlatu").as_deref(), Some("mlat"));
        assert_eq!(implicit_four_letter_gismu_rafsi("broda"), None);
        assert_eq!(implicit_four_letter_gismu_rafsi("blan"), None);
        assert_eq!(implicit_four_letter_gismu_rafsi("blany"), None);
    }

    #[test]
    fn test_create_every_possibility_cartesian() {
        let input = vec![vec!["a", "b"], vec!["1", "2"]];
        let mut got = cartesian_product(input);
        got.sort();
        let mut expected = vec![
            vec!["a", "1"],
            vec!["a", "2"],
            vec!["b", "1"],
            vec!["b", "2"],
        ];
        expected.sort();
        assert_eq!(got, expected);
    }

    #[test]
    fn test_reconstruct_lujvo_basic() {
        let options = RafsiOptions {
            exp_rafsi: false,
            custom_cmavo: None,
            custom_cmavo_exp: None,
            custom_gismu: None,
            custom_gismu_exp: None,
        };
        let result = reconstruct_lujvo("bramlatu", true, &options).unwrap();
        assert_eq!(result, "bramlatu");
        let result = reconstruct_lujvo("bardymlatu", true, &options).unwrap();
        assert_eq!(result, "bramlatu");
    }

    #[test]
    fn test_reconstruct_lujvo_with_y_hyphen() {
        let options = RafsiOptions {
            exp_rafsi: false,
            custom_cmavo: None,
            custom_cmavo_exp: None,
            custom_gismu: None,
            custom_gismu_exp: None,
        };
        let result = reconstruct_lujvo("klamyseltru", true, &options).unwrap();
        assert_eq!(result, "klaseltru");
    }

    #[test]
    fn test_reconstruct_lujvo_with_r_hyphen() {
        let options = RafsiOptions {
            exp_rafsi: false,
            custom_cmavo: None,
            custom_cmavo_exp: None,
            custom_gismu: None,
            custom_gismu_exp: None,
        };
        let result = reconstruct_lujvo("toirbroda", true, &options).unwrap();
        assert_eq!(result, "toirbroda");
    }

    #[test]
    fn test_reconstruct_lujvo_with_apostrophe() {
        let options = RafsiOptions {
            exp_rafsi: false,
            custom_cmavo: None,
            custom_cmavo_exp: None,
            custom_gismu: None,
            custom_gismu_exp: None,
        };
        let result = reconstruct_lujvo("ca'irgau", true, &options).unwrap();
        assert_eq!(result, "ca'irgau");
    }

    #[test]
    fn test_reconstruct_invalid_lujvo() {
        let options = RafsiOptions {
            exp_rafsi: false,
            custom_cmavo: None,
            custom_cmavo_exp: None,
            custom_gismu: None,
            custom_gismu_exp: None,
        };
        assert!(reconstruct_lujvo("invalid", false, &options).is_err());
    }

    #[test]
    fn test_reconstruct_empty_string() {
        let options = RafsiOptions {
            exp_rafsi: false,
            custom_cmavo: None,
            custom_cmavo_exp: None,
            custom_gismu: None,
            custom_gismu_exp: None,
        };
        assert!(reconstruct_lujvo("", false, &options).is_err());
    }

    #[test]
    fn test_search_selrafsi_cvcc_picks_real_gismu() {
        let options = RafsiOptions {
            exp_rafsi: false,
            custom_cmavo: None,
            custom_cmavo_exp: None,
            custom_gismu: None,
            custom_gismu_exp: None,
        };
        assert_eq!(
            resolve_selrafsi("zukt", &options),
            Some("zukte".to_string())
        );
    }

    #[test]
    fn test_search_selrafsi_uses_custom_gismu_for_reverse_lookup() {
        let mut custom_gismu: HashMap<String, Vec<String>> = HashMap::new();
        custom_gismu.insert("klum".into(), vec!["klu".into()]);
        let options = RafsiOptions {
            exp_rafsi: false,
            custom_cmavo: None,
            custom_cmavo_exp: None,
            custom_gismu: Some(&custom_gismu),
            custom_gismu_exp: None,
        };
        assert_eq!(
            resolve_selrafsi("klu", &options),
            Some("klum".to_string())
        );
        assert_eq!(
            resolve_selrafsi("zuk", &options),
            Some("zukte".to_string())
        );
    }

    #[test]
    fn test_search_selrafsi_four_letter_falls_through_for_empty_rafsi_gismu() {
        let mut custom_gismu: HashMap<String, Vec<String>> = HashMap::new();
        custom_gismu.insert("vreji".into(), vec!["vei".into()]);
        let options = RafsiOptions {
            exp_rafsi: true,
            custom_cmavo: None,
            custom_cmavo_exp: None,
            custom_gismu: Some(&custom_gismu),
            custom_gismu_exp: None,
        };
        assert_eq!(
            resolve_selrafsi("datn", &options),
            Some("datni".to_string())
        );
    }

    #[test]
    fn test_reconstruct_zuktydea_to_zukydea() {
        let options = RafsiOptions {
            exp_rafsi: true,
            custom_cmavo: None,
            custom_cmavo_exp: None,
            custom_gismu: None,
            custom_gismu_exp: None,
        };
        assert_eq!(
            reconstruct_lujvo("zuktyde'a", true, &options).unwrap(),
            "zukyde'a"
        );
    }

    #[test]
    fn test_reconstruct_falls_back_to_builtins_with_incomplete_custom_maps() {
        let mut custom_gismu: HashMap<String, Vec<String>> = HashMap::new();
        custom_gismu.insert("broda".into(), vec!["rod".into(), "brod".into()]);
        let options = RafsiOptions {
            exp_rafsi: true,
            custom_cmavo: None,
            custom_cmavo_exp: None,
            custom_gismu: Some(&custom_gismu),
            custom_gismu_exp: None,
        };
        assert_eq!(
            reconstruct_lujvo("rivyzu'e", true, &options).unwrap(),
            "rivzu'e"
        );
    }

    #[test]
    fn test_reconstruct_does_not_drop_unresolved_rafsi() {
        let mut custom_gismu: HashMap<String, Vec<String>> = HashMap::new();
        custom_gismu.insert("vreji".into(), vec!["vei".into()]);
        custom_gismu.insert("liste".into(), vec!["ste".into(), "list".into()]);
        let empty_exp = HashMap::new();
        let options = RafsiOptions {
            exp_rafsi: true,
            custom_cmavo: None,
            custom_cmavo_exp: None,
            custom_gismu: Some(&custom_gismu),
            custom_gismu_exp: Some(&empty_exp),
        };
        assert_eq!(
            reconstruct_lujvo("datnyveiste", true, &options).unwrap(),
            "datnyveiste"
        );
        let a = analyze_lujvo_spelling("datnyveiste", &options).unwrap();
        assert!(a.is_score_optimal);
        assert_eq!(a.canonical, "datnyveiste");
    }

    #[test]
    fn test_analyze_lujvo_spelling_optional_extra_y() {
        let options = RafsiOptions {
            exp_rafsi: true,
            custom_cmavo: None,
            custom_cmavo_exp: None,
            custom_gismu: None,
            custom_gismu_exp: None,
        };
        let a = analyze_lujvo_spelling("rivyzu'e", &options).unwrap();
        assert!(!a.is_score_optimal);
        assert_eq!(a.canonical, "rivzu'e");
        let b = analyze_lujvo_spelling("rivzu'e", &options).unwrap();
        assert!(b.is_score_optimal);
        assert_eq!(b.canonical, "rivzu'e");
    }

    #[test]
    fn test_analyze_lujvo_spelling_score_suboptimal_rafsi() {
        let options = RafsiOptions {
            exp_rafsi: true,
            custom_cmavo: None,
            custom_cmavo_exp: None,
            custom_gismu: None,
            custom_gismu_exp: None,
        };
        let a = analyze_lujvo_spelling("bardymlatu", &options).unwrap();
        assert!(!a.is_score_optimal);
        assert_eq!(a.canonical, "bramlatu");
    }

    #[test]
    fn test_search_selrafsi_empty_rafsi_list_gismu() {
        let options = RafsiOptions {
            exp_rafsi: false,
            custom_cmavo: None,
            custom_cmavo_exp: None,
            custom_gismu: None,
            custom_gismu_exp: None,
        };
        assert_eq!(
            resolve_selrafsi("karce", &options),
            Some("karce".to_string())
        );
        assert_eq!(
            gismu_rafsi_list("karce", false, None, None),
            Some(Vec::<String>::new())
        );
    }

    #[test]
    fn test_reconstruct_sorprekarce_keeps_karce() {
        let options = RafsiOptions {
            exp_rafsi: false,
            custom_cmavo: None,
            custom_cmavo_exp: None,
            custom_gismu: None,
            custom_gismu_exp: None,
        };
        assert_eq!(
            reconstruct_lujvo("sorprekarce", true, &options).unwrap(),
            "sorprekarce"
        );
        let a = analyze_lujvo_spelling("sorprekarce", &options).unwrap();
        assert!(a.is_score_optimal);
        assert_eq!(a.canonical, "sorprekarce");
    }

    #[cfg(feature = "camxes")]
    #[test]
    fn test_reconstruct_valsykrakatu() {
        let options = RafsiOptions {
            exp_rafsi: false,
            custom_cmavo: None,
            custom_cmavo_exp: None,
            custom_gismu: None,
            custom_gismu_exp: None,
        };
        assert_eq!(reconstruct_lujvo("valsykrakatu", true, &options).unwrap(), "valykrakatu");
        assert_eq!(reconstruct_lujvo("tci'ilyfinpe", true, &options).unwrap(), "tci'ilyfi'e");
        let analysis = analyze_lujvo_spelling("valsykrakatu", &options).unwrap();
        assert_eq!(analysis.canonical, "valykrakatu");
        assert!(!analysis.is_score_optimal);
        assert!(analyze_lujvo_spelling("valykrakatu", &options).unwrap().is_score_optimal);
    }
}
