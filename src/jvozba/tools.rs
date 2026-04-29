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
    if let Some(custom_gismu) = custom_gismu {
        if let Some(rafsi) = custom_gismu.get(a) {
            if !rafsi.is_empty() {
                return Some(rafsi.clone());
            }
        }
    } else if let Some(rafsi) = get_gismu_rafsi_list().get(a) {
        if !rafsi.is_empty() {
            return Some(rafsi.clone());
        }
    }

    if exp_rafsi {
        if let Some(custom_gismu_exp) = custom_gismu_exp {
            if let Some(rafsi) = custom_gismu_exp.get(a) {
                if !rafsi.is_empty() {
                    return Some(rafsi.clone());
                }
            }
        } else if let Some(rafsi) = get_gismu_rafsi_list_exp().get(a) {
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
            if !rafsi.is_empty() {
                return Some(rafsi.clone());
            }
        }
    } else if let Some(rafsi) = get_cmavo_rafsi_list().get(a) {
        if !rafsi.is_empty() {
            return Some(rafsi.clone());
        }
    }

    if exp_rafsi {
        if let Some(custom_cmavo_exp) = custom_cmavo_exp {
            if let Some(rafsi) = custom_cmavo_exp.get(a) {
                if !rafsi.is_empty() {
                    return Some(rafsi.clone());
                }
            }
        } else if let Some(rafsi) = get_cmavo_rafsi_list_exp().get(a) {
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
/// * `exp_rafsi` - Whether to use experimental rafsi
///
/// # Returns
/// Result with reconstructed lujvo or error message
pub fn reconstruct_lujvo(
    lujvo: &str,
    forbid_cmevla: bool,
    options: &RafsiOptions,
) -> Result<String, Box<dyn std::error::Error>> {
    // Split into rafsi
    let rafsi_list = jvokaha::jvokaha(lujvo)?;

    // Get selrafsi for each rafsi
    let selrafsi_list: Vec<String> = rafsi_list
        .iter()
        .filter_map(|rafsi| {
            if rafsi == "y" || rafsi == "r" || rafsi == "n" {
                None
            } else {
                search_selrafsi_from_rafsi2(
                    rafsi,
                    options,
                )
            }
        })
        .collect();

    // Rebuild using jvozba
    let rebuilt = narge::jvozba(
        &selrafsi_list,
        false,
        forbid_cmevla,
        options,
    )
    .first()
    .ok_or("Failed to rebuild lujvo")?
    .lujvo
    .clone();

    Ok(rebuilt)
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

    /// When custom_gismu is provided it must fully replace the bundled JSON
    /// for the reverse rafsi->selrafsi lookup.
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
        // "klm" only exists in the custom map -> must resolve to "klamb"
        assert_eq!(
            search_selrafsi_from_rafsi2("klu", &options),
            Some("klum".to_string())
        );
        // Bundled-only rafsi must NOT be found when a custom map is supplied
        assert_eq!(search_selrafsi_from_rafsi2("zuk", &options), None);
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
}

pub fn search_selrafsi_from_rafsi2(
    rafsi: &str,
    options: &RafsiOptions,
) -> Option<String> {
    if let Some(rafsis) = gismu_rafsi_list(rafsi, options.exp_rafsi, options.custom_gismu, options.custom_gismu_exp) {
        if !rafsis.is_empty() {
            return Some(rafsi.to_owned());
        }
    }

    if rafsi != "brod" && rafsi.len() == 4 && !rafsi.contains('\'') {
        for vowel in "aeiou".chars() {
            let gismu_candid = format!("{}{}", rafsi, vowel);
            let exists = match options.custom_gismu {
                Some(m) => m.contains_key(&gismu_candid),
                None => get_gismu_rafsi_list().contains_key(&gismu_candid),
            } || (options.exp_rafsi
                && match options.custom_gismu_exp {
                    Some(m) => m.contains_key(&gismu_candid),
                    None => get_gismu_rafsi_list_exp().contains_key(&gismu_candid),
                });
            if exists {
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

    if let Some(found) = match options.custom_gismu {
        Some(m) => find_in(m),
        None => find_in(get_gismu_rafsi_list()),
    } {
        return Some(found);
    }

    if let Some(found) = match options.custom_cmavo {
        Some(m) => find_in(m),
        None => find_in(get_cmavo_rafsi_list()),
    } {
        return Some(found);
    }

    if options.exp_rafsi {
        if let Some(found) = match options.custom_gismu_exp {
            Some(m) => find_in(m),
            None => find_in(get_gismu_rafsi_list_exp()),
        } {
            return Some(found);
        }

        if let Some(found) = match options.custom_cmavo_exp {
            Some(m) => find_in(m),
            None => find_in(get_cmavo_rafsi_list_exp()),
        } {
            return Some(found);
        }
    }

    None
}
