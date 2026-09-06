use std::collections::HashMap;

use serde_json;

use super::rafsi_list::{
    get_cmavo_rafsi_list, get_cmavo_rafsi_list_exp, get_gismu_rafsi_list, get_gismu_rafsi_list_exp,
};
use super::{jvokaha, narge};

#[derive(Clone)]
pub struct RafsiOptions<'a> {
    pub exp_rafsi: bool,
    pub custom_cmavo: Option<&'a HashMap<String, Vec<String>>>,
    pub custom_cmavo_exp: Option<&'a HashMap<String, Vec<String>>>,
    pub custom_gismu: Option<&'a HashMap<String, Vec<String>>>,
    pub custom_gismu_exp: Option<&'a HashMap<String, Vec<String>>>,
}

pub fn create_every_possibility<T: Clone + serde::Serialize + for<'de> serde::Deserialize<'de>>(
    aa: Vec<Vec<T>>,
) -> Vec<Vec<T>> {
    let mut arr_arr: Vec<Vec<T>> =
        serde_json::from_str(&serde_json::to_string(&aa).unwrap()).unwrap();
    if arr_arr.is_empty() {
        return vec![vec![]];
    }
    let arr = arr_arr.pop().unwrap();

    let mut result: Vec<Vec<T>> = Vec::new();
    for e in arr {
        let sub_results = create_every_possibility(arr_arr.clone());
        for mut f in sub_results {
            f.push(e.clone());
            result.push(f);
        }
    }
    result
}

pub fn gismu_rafsi_list(
    a: &str,
    exp_rafsi: bool,
    custom_gismu: Option<&HashMap<String, Vec<String>>>,
    custom_gismu_exp: Option<&HashMap<String, Vec<String>>>,
) -> Option<Vec<String>> {
    // Custom map overrides per key; missing keys fall through to builtins.
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

pub fn get_candid(selrafsi: &str, is_last: bool, options: &RafsiOptions) -> Vec<String> {
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

        let chopped = gismu
            .chars()
            .take(gismu.chars().count() - 1)
            .collect::<String>();
        if chopped != "brod" {
            candid.push(chopped);
        }
        candid
    } else {
        Vec::new()
    }
}

/// Reconstruct a lujvo from its components
///
/// # Arguments
/// * `lujvo` - The lujvo to reconstruct
/// * `forbid_cmevla` - Whether to forbid cmevla in the rebuild
/// * `options` - Rafsi lookup options (custom maps and experimental rafsi)
///
/// # Returns
/// Result with reconstructed lujvo or error message
///
/// When custom rafsi maps are incomplete, missing keys fall through to the
/// built-in lists (per-key override, not wholesale replace). A full builtin
/// retry is kept for other reconstruct failures.
pub fn reconstruct_lujvo(
    lujvo: &str,
    forbid_cmevla: bool,
    options: &RafsiOptions,
) -> Result<String, Box<dyn std::error::Error>> {
    match reconstruct_lujvo_with(lujvo, forbid_cmevla, options) {
        Ok(s) => Ok(s),
        Err(e) => {
            let has_custom = options.custom_cmavo.is_some()
                || options.custom_cmavo_exp.is_some()
                || options.custom_gismu.is_some()
                || options.custom_gismu_exp.is_some();
            if !has_custom {
                return Err(e);
            }
            let builtin = RafsiOptions {
                exp_rafsi: options.exp_rafsi,
                custom_cmavo: None,
                custom_cmavo_exp: None,
                custom_gismu: None,
                custom_gismu_exp: None,
            };
            reconstruct_lujvo_with(lujvo, forbid_cmevla, &builtin)
        }
    }
}

fn reconstruct_lujvo_with(
    lujvo: &str,
    forbid_cmevla: bool,
    options: &RafsiOptions,
) -> Result<String, Box<dyn std::error::Error>> {
    let rafsi_list = jvokaha::jvokaha(lujvo)?;

    // Every non-hyphen piece must resolve. Silently dropping unknowns (via
    // filter_map) used to rebuild a shorter lujvo as Ok (e.g. datnyveiste →
    // veiste when `datn` failed reverse-lookup under incomplete custom maps).
    let mut selrafsi_list = Vec::new();
    for rafsi in &rafsi_list {
        if rafsi == "y" || rafsi == "r" || rafsi == "n" {
            continue;
        }
        let Some(selrafsi) = search_selrafsi_from_rafsi2(rafsi, options) else {
            return Err(format!("Could not resolve rafsi `{rafsi}` in `{lujvo}`").into());
        };
        selrafsi_list.push(selrafsi);
    }

    if selrafsi_list.len() < 2 {
        return Err("Need at least two selrafsi to rebuild lujvo".into());
    }

    let rebuilt = narge::jvozba(&selrafsi_list, false, forbid_cmevla, true, options)
        .first()
        .ok_or("Failed to rebuild lujvo")?
        .lujvo
        .clone();

    Ok(rebuilt)
}

/// Score-optimal spelling analysis for a classical lujvo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LujvoSpellingAnalysis {
    /// Best-scoring form from the same selrafsi (`reconstruct_lujvo`).
    pub canonical: String,
    /// True when `canonical` equals the input spelling.
    pub is_score_optimal: bool,
}

/// Compare a lujvo spelling to its score-optimal form.
///
/// Returns `None` if the string is not a classical lujvo (`jvokaha` /
/// `reconstruct_lujvo` fails even with built-in rafsi lists).
pub fn analyze_lujvo_spelling(
    word: &str,
    options: &RafsiOptions,
) -> Option<LujvoSpellingAnalysis> {
    jvokaha::jvokaha(word).ok()?;
    let canonical = reconstruct_lujvo(word, true, options).ok()?;
    Some(LujvoSpellingAnalysis {
        is_score_optimal: canonical == word,
        canonical,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let options = RafsiOptions {
            exp_rafsi: false,
            custom_cmavo: None,
            custom_cmavo_exp: None,
            custom_gismu: None,
            custom_gismu_exp: None,
        };
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

    /// Regression: search_selrafsi_from_rafsi2 must look up the actual gismu
    /// key, not just the first vowel that "exists". Previously it returned
    /// "zukta" (not a gismu) for "zukt" because gismu_rafsi_list always
    /// returns Some(empty_vec) for any string.
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
            search_selrafsi_from_rafsi2("zukt", &options),
            Some("zukte".to_string())
        );
    }

    /// Custom keys override builtins; missing keys fall through.
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
            search_selrafsi_from_rafsi2("klu", &options),
            Some("klum".to_string())
        );
        // Not in custom map → builtin fallthrough (zuk ← zukte)
        assert_eq!(
            search_selrafsi_from_rafsi2("zuk", &options),
            Some("zukte".to_string())
        );
    }

    /// datni has no assigned short rafsi (empty list / DB NULL) but `datn` is
    /// still the legal 4-letter form. Incomplete custom maps that omit datni
    /// must still resolve via builtin keys (including empty-rafsi gismu).
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
            search_selrafsi_from_rafsi2("datn", &options),
            Some("datni".to_string())
        );
    }

    /// Regression: reconstructing "zuktyde'a" (CVCC rafsi of zukte + y + de'a)
    /// with forbid_cmevla=true must propose the better "zukyde'a" variant.
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

    /// Regression: dropping unresolved rafsi used to rebuild a shorter lujvo
    /// (datnyveiste → veiste) as Ok, skipping the builtin fallback.
    #[test]
    fn test_reconstruct_does_not_drop_unresolved_rafsi() {
        let mut custom_gismu: HashMap<String, Vec<String>> = HashMap::new();
        custom_gismu.insert("vreji".into(), vec!["vei".into()]);
        custom_gismu.insert("liste".into(), vec!["ste".into(), "list".into()]);
        // omit datni — previously yielded Ok("veiste")
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
}

pub fn search_selrafsi_from_rafsi2(
    rafsi: &str,
    options: &RafsiOptions,
) -> Option<String> {
    if let Some(rafsis) = gismu_rafsi_list(
        rafsi,
        options.exp_rafsi,
        options.custom_gismu,
        options.custom_gismu_exp,
    ) {
        if !rafsis.is_empty() {
            return Some(rafsi.to_owned());
        }
    }

    if rafsi != "brod" && rafsi.len() == 4 && !rafsi.contains('\'') {
        for vowel in "aeiou".chars() {
            let gismu_candid = format!("{}{}", rafsi, vowel);
            if gismu_key_exists(&gismu_candid, options) {
                return Some(gismu_candid);
            }
        }
    }

    let needle = rafsi.to_string();
    let find_in = |map: &HashMap<String, Vec<String>>| -> Option<String> {
        map.iter()
            .find(|(_, list)| list.contains(&needle))
            .map(|(k, _)| k.clone())
    };

    // Prefer custom maps, then fall through to builtins for missing keys.
    if let Some(m) = options.custom_gismu {
        if let Some(found) = find_in(m) {
            return Some(found);
        }
    }
    if let Some(found) = find_in(get_gismu_rafsi_list()) {
        return Some(found);
    }

    if let Some(m) = options.custom_cmavo {
        if let Some(found) = find_in(m) {
            return Some(found);
        }
    }
    if let Some(found) = find_in(get_cmavo_rafsi_list()) {
        return Some(found);
    }

    if options.exp_rafsi {
        if let Some(m) = options.custom_gismu_exp {
            if let Some(found) = find_in(m) {
                return Some(found);
            }
        }
        if let Some(found) = find_in(get_gismu_rafsi_list_exp()) {
            return Some(found);
        }

        if let Some(m) = options.custom_cmavo_exp {
            if let Some(found) = find_in(m) {
                return Some(found);
            }
        }
        if let Some(found) = find_in(get_cmavo_rafsi_list_exp()) {
            return Some(found);
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
