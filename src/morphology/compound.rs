use super::{
    cv::{CvInfo, cv_shape, is_consonant},
    lookup::{RafsiOptions, cartesian_product, rafsi_candidates},
    score::lujvo_score,
};
use crate::error::{Result, VlazbaError};
use serde_json::Value;
use std::collections::{HashMap, VecDeque};
use std::sync::LazyLock;

/// 128×128 ASCII permissibility table: 0 = forbidden, 1/2 = allowed grades.
static PERMISSIBILITY: LazyLock<[[i8; 128]; 128]> = LazyLock::new(|| {
    let json: Value = serde_json::from_str(include_str!("permissible.json"))
        .expect("Invalid JSON in permissible.json");
    let mut table = [[0i8; 128]; 128];
    for (k, v) in json.as_object().expect("permissible root object") {
        let c1 = k.chars().next().expect("row key") as u8;
        for (k2, v2) in v.as_object().expect("permissible row") {
            let c2 = k2.chars().next().expect("col key") as u8;
            table[c1 as usize][c2 as usize] = v2.as_i64().expect("perm value") as i8;
        }
    }
    table
});

#[inline]
fn is_permissible(c1: char, c2: char) -> i32 {
    let (a, b) = (c1 as u32, c2 as u32);
    if a < 128 && b < 128 {
        i32::from(PERMISSIBILITY[a as usize][b as usize])
    } else {
        0
    }
}

#[derive(Debug, Clone)]
pub struct LujvoAndScore {
    pub lujvo: String,
    pub score: i32,
}

/// Options for [`jvozba_with`].
#[derive(Clone, Copy)]
pub struct JvozbaOptions<'a> {
    pub forbid_la_lai_doi: bool,
    pub forbid_cmevla: bool,
    /// If true, return only minimum-score forms via DP (no full Cartesian product).
    pub best_only: bool,
    pub rafsi: &'a RafsiOptions<'a>,
}

/// Generate possible lujvo combinations from a list of selrafsi.
///
/// Prefer [`jvozba_with`] when constructing options explicitly. `best_only`
/// is recommended for score-optimal forms (used by reconstruct).
pub fn jvozba(
    selrafsi: &[String],
    forbid_la_lai_doi: bool,
    forbid_cmevla: bool,
    best_only: bool,
    options: &RafsiOptions,
) -> Vec<LujvoAndScore> {
    jvozba_with(
        selrafsi,
        &JvozbaOptions {
            forbid_la_lai_doi,
            forbid_cmevla,
            best_only,
            rafsi: options,
        },
    )
}

/// Generate lujvo using a single options struct.
pub fn jvozba_with(selrafsi: &[String], opts: &JvozbaOptions<'_>) -> Vec<LujvoAndScore> {
    if opts.best_only {
        return jvozba_best_only(selrafsi, opts);
    }

    let rafsi_choices: Vec<Vec<String>> = selrafsi
        .iter()
        .enumerate()
        .map(|(i, word)| rafsi_candidates(word, i + 1 == selrafsi.len(), opts.rafsi))
        .collect();

    let mut results: Vec<LujvoAndScore> = cartesian_product(rafsi_choices)
        .into_iter()
        .filter_map(|rafsi_list| {
            normalize(&rafsi_list).ok().map(|parts| LujvoAndScore {
                lujvo: parts.join(""),
                score: lujvo_score(&parts),
            })
        })
        .filter(|candidate| {
            !is_forbidden(candidate, opts.forbid_la_lai_doi)
                && !(opts.forbid_cmevla && is_cmevla(&candidate.lujvo))
        })
        .collect();

    results.sort_unstable_by(|a, b| a.score.cmp(&b.score).then_with(|| a.lujvo.cmp(&b.lujvo)));
    results
}

/// Right-to-left DP for score-optimal lujvo only.
///
/// # Correctness sketch
/// CLL score is additive over the normalized rafsi/hyphen sequence. Hyphens
/// between rafsi `i` and `i+1` for `i > 0` depend only on those two rafsi.
/// The first rafsi additionally needs the tosmabru / CVV-r rules, which read
/// only: whether the final form is a cmevla, the head component, and the
/// leading CVC-run through the first non-CVC (`tosmabru_prefix`).
///
/// Therefore any two suffixes with the same [`SuffixInterface`] have identical
/// future hyphen behaviour for every left extension; keeping only the
/// minimum-score suffix(es) per interface never drops a globally optimal
/// completion. Final answers are exactly the forms whose score equals the
/// global minimum (ties retained).
fn jvozba_best_only(selrafsi: &[String], opts: &JvozbaOptions<'_>) -> Vec<LujvoAndScore> {
    let n = selrafsi.len();
    if n < 2 {
        return Vec::new();
    }

    let rafsi_choices: Vec<Vec<String>> = selrafsi
        .iter()
        .enumerate()
        .map(|(i, word)| rafsi_candidates(word, i + 1 == selrafsi.len(), opts.rafsi))
        .collect();

    if rafsi_choices.iter().any(|c| c.is_empty()) {
        return Vec::new();
    }

    let mut layer: HashMap<SuffixInterface, (i32, Vec<Vec<String>>)> = HashMap::new();
    for rafsi in &rafsi_choices[n - 1] {
        let components = vec![rafsi.clone()];
        let score = lujvo_score(&components);
        insert_best_suffix(&mut layer, suffix_interface(&components), score, components);
    }

    for i in (0..n - 1).rev() {
        let mut next_layer: HashMap<SuffixInterface, (i32, Vec<Vec<String>>)> = HashMap::new();
        let is_first = i == 0;
        for rafsi in &rafsi_choices[i] {
            for (_suf_score, seqs) in layer.values() {
                for suffix in seqs {
                    let joined = attach_left(rafsi, suffix, is_first, n);
                    let score = lujvo_score(&joined);
                    if is_first {
                        let candidate = LujvoAndScore {
                            lujvo: joined.join(""),
                            score,
                        };
                        if is_forbidden(&candidate, opts.forbid_la_lai_doi)
                            || (opts.forbid_cmevla && is_cmevla(&candidate.lujvo))
                        {
                            continue;
                        }
                    }
                    insert_best_suffix(
                        &mut next_layer,
                        suffix_interface(&joined),
                        score,
                        joined,
                    );
                }
            }
        }
        layer = next_layer;
    }

    let mut results: Vec<LujvoAndScore> = layer
        .into_values()
        .flat_map(|(score, seqs)| {
            seqs.into_iter().map(move |components| LujvoAndScore {
                lujvo: components.join(""),
                score,
            })
        })
        .collect();

    if let Some(min_score) = results.iter().map(|a| a.score).min() {
        results.retain(|a| a.score == min_score);
    }
    results.sort_unstable_by(|a, b| a.score.cmp(&b.score).then_with(|| a.lujvo.cmp(&b.lujvo)));
    results.dedup_by(|a, b| a.lujvo == b.lujvo);
    results
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
struct SuffixInterface {
    head: String,
    cmevla: bool,
    tosmabru_prefix: Vec<String>,
}

fn suffix_interface(components: &[String]) -> SuffixInterface {
    let head = components[0].clone();
    let cmevla = is_cmevla(components.last().expect("non-empty suffix"));
    let tosmabru_prefix = match components.iter().position(|s| !is_cvc(s)) {
        Some(i) => components[..=i].to_vec(),
        None => Vec::new(),
    };
    SuffixInterface {
        head,
        cmevla,
        tosmabru_prefix,
    }
}

fn insert_best_suffix(
    layer: &mut HashMap<SuffixInterface, (i32, Vec<Vec<String>>)>,
    iface: SuffixInterface,
    score: i32,
    components: Vec<String>,
) {
    match layer.get_mut(&iface) {
        None => {
            layer.insert(iface, (score, vec![components]));
        }
        Some((best, seqs)) => {
            if score < *best {
                *best = score;
                seqs.clear();
                seqs.push(components);
            } else if score == *best && !seqs.contains(&components) {
                seqs.push(components);
            }
        }
    }
}

fn attach_left(
    rafsi: &str,
    suffix: &[String],
    is_first: bool,
    total_rafsi_count: usize,
) -> Vec<String> {
    let mut result: VecDeque<String> = suffix.iter().cloned().collect();
    let end = rafsi.chars().last().expect("non-empty rafsi");
    let init = result[0].chars().next().expect("non-empty suffix");

    let y_inserted = if is_4letter(rafsi)
        || (is_consonant(end) && is_consonant(init) && is_permissible(end, init) == 0)
        || (end == 'n'
            && ["ts", "tc", "dz", "dj"]
                .iter()
                .any(|&s| result[0].starts_with(s)))
    {
        result.push_front("y".to_string());
        true
    } else {
        false
    };

    if is_first && is_cvv(rafsi) {
        let hyphen = if result[0].starts_with('r') { "n" } else { "r" };
        if total_rafsi_count > 2 || !is_ccv(&result[0]) {
            result.push_front(hyphen.to_string());
        }
    } else if is_first && !y_inserted && is_cvc(rafsi) && is_tosmabru(rafsi, result.make_contiguous())
    {
        result.push_front("y".to_string());
    }

    result.push_front(rafsi.to_string());
    result.into()
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
    valsi.chars().last().is_some_and(is_consonant)
}

pub fn normalize(rafsi_list: &[String]) -> Result<Vec<String>> {
    if rafsi_list.len() < 2 {
        return Err(VlazbaError::TooFewValsi);
    }

    let mut result: VecDeque<String> = VecDeque::with_capacity(rafsi_list.len() * 2 - 1);
    result.push_back(rafsi_list.last().unwrap().clone());

    for (i, rafsi) in rafsi_list.iter().rev().skip(1).enumerate() {
        let end = rafsi.chars().last().unwrap();
        let init = result[0].chars().next().unwrap();

        let y_inserted = if is_4letter(rafsi)
            || (is_consonant(end) && is_consonant(init) && is_permissible(end, init) == 0)
            || (end == 'n'
                && ["ts", "tc", "dz", "dj"]
                    .iter()
                    .any(|&s| result[0].starts_with(s)))
        {
            result.push_front("y".to_string());
            true
        } else {
            false
        };

        if i == rafsi_list.len() - 2 && is_cvv(rafsi) {
            let hyphen = if result[0].starts_with('r') { "n" } else { "r" };
            if rafsi_list.len() > 2 || !is_ccv(&result[0]) {
                result.push_front(hyphen.to_string());
            }
        } else if !y_inserted
            && i == rafsi_list.len() - 2
            && is_cvc(rafsi)
            && is_tosmabru(rafsi, result.make_contiguous())
        {
            result.push_front("y".to_string());
        }

        result.push_front(rafsi.clone());
    }

    Ok(result.into())
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
            && (cv_shape(s) != CvInfo::CVCCV
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
    cv_shape(rafsi).is_cvv()
}

#[inline]
fn is_ccv(rafsi: &str) -> bool {
    cv_shape(rafsi).is_ccv()
}

#[inline]
fn is_cvc(rafsi: &str) -> bool {
    cv_shape(rafsi).is_cvc()
}

#[inline]
fn is_4letter(rafsi: &str) -> bool {
    cv_shape(rafsi).is_4letter()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_options() -> RafsiOptions<'static> {
        RafsiOptions {
            exp_rafsi: true,
            custom_cmavo: None,
            custom_cmavo_exp: None,
            custom_gismu: None,
            custom_gismu_exp: None,
        }
    }

    #[test]
    fn test_jvozba_klama_gasnu() {
        let input = vec!["klama".to_string(), "gasnu".to_string()];
        let options = default_options();
        let result = jvozba(&input, false, false, false, &options);

        assert!(
            !result.is_empty(),
            "jvozba should return at least one result"
        );
        assert_eq!(result[0].lujvo, "klagau", "First result should be 'klagau'");
    }

    #[test]
    fn test_jvozba_single_word() {
        let input = vec!["klama".to_string()];
        let result = jvozba(&input, false, false, false, &default_options());
        assert!(result.is_empty(), "Single word should return empty result");
    }

    #[test]
    fn test_jvozba_empty_input() {
        let input: Vec<String> = vec![];
        let result = jvozba(&input, false, false, false, &default_options());
        assert!(result.is_empty(), "Empty input should return empty result");
    }

    #[test]
    fn test_jvozba_experimental_rafsi() {
        let input = vec!["klama".to_string(), "gasnu".to_string()];
        let result = jvozba(&input, false, false, false, &default_options());
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

        let result = jvozba(&input, false, false, false, &options);
        assert!(!result.is_empty(), "Should use custom gismu rafsi");
        assert!(
            result.iter().any(|r| r.lujvo == "qlagasnu"),
            "Expected custom rafsi combination"
        );
    }

    #[test]
    fn test_jvozba_best_only_matches_full_enumeration() {
        let options = default_options();
        let cases = [
            vec!["klama".into(), "gasnu".into()],
            vec!["blanu".into(), "zdani".into()],
            vec!["melbi".into(), "cmalu".into(), "zdani".into()],
            vec!["broda".into(), "brode".into()],
            vec!["zukte".into(), "denpa".into()],
            vec!["tirxu".into(), "broda".into()],
            vec!["cmavo".into(), "gasnu".into()],
        ];
        for input in cases {
            let full = jvozba(&input, false, true, false, &options);
            let best = jvozba(&input, false, true, true, &options);
            assert!(!full.is_empty(), "full enumeration empty for {input:?}");
            let min = full[0].score;
            let full_best: Vec<_> = full
                .iter()
                .filter(|r| r.score == min)
                .map(|r| r.lujvo.clone())
                .collect();
            let dp_best: Vec<_> = best.iter().map(|r| r.lujvo.clone()).collect();
            assert_eq!(
                full_best, dp_best,
                "best-only mismatch for {input:?}: full={full_best:?} dp={dp_best:?}"
            );
            assert!(
                best.iter().all(|r| r.score == min),
                "best-only returned non-minimal score for {input:?}"
            );
        }
    }

    #[test]
    fn test_jvozba_best_only_klagau() {
        let options = default_options();
        let input = vec!["klama".to_string(), "gasnu".to_string()];
        let best = jvozba(&input, false, false, true, &options);
        assert_eq!(best[0].lujvo, "klagau");
        assert!(best.iter().all(|r| r.score == best[0].score));
    }

    #[test]
    fn test_jvozba_with_options_struct() {
        let options = default_options();
        let input = vec!["klama".to_string(), "gasnu".to_string()];
        let via_with = jvozba_with(
            &input,
            &JvozbaOptions {
                forbid_la_lai_doi: false,
                forbid_cmevla: false,
                best_only: true,
                rafsi: &options,
            },
        );
        assert_eq!(via_with[0].lujvo, "klagau");
    }

    #[test]
    fn test_attach_left_matches_normalize() {
        let lists = [
            vec!["kla".into(), "gau".into()],
            vec!["zuk".into(), "de'a".into()],
            vec!["slak".into(), "gau".into()],
            vec!["bra".into(), "mlatu".into()],
            vec!["toi".into(), "broda".into()],
            vec!["sel".into(), "kla".into(), "gau".into()],
        ];
        for list in lists {
            let via_normalize = normalize(&list).unwrap();
            let mut acc = vec![list.last().unwrap().clone()];
            for (rev_i, rafsi) in list.iter().rev().skip(1).enumerate() {
                let is_first = rev_i == list.len() - 2;
                acc = attach_left(rafsi, &acc, is_first, list.len());
            }
            assert_eq!(acc, via_normalize, "attach_left diverged for {list:?}");
        }
    }

    #[test]
    fn test_is_tosmabru() {
        let rafsi = "tos";
        let rest = vec!["mabru".to_string()];
        assert!(
            is_tosmabru(rafsi, &rest),
            "'tosmabru' should be a valid tosmabru"
        );

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
        assert_eq!(result.unwrap_err(), VlazbaError::TooFewValsi);
    }

    #[test]
    fn test_is_cmevla() {
        assert!(is_cmevla("klaman"), "Should recognize cmevla");
        assert!(!is_cmevla("klama"), "Should recognize non-cmevla");
    }

    #[test]
    fn test_normalize_cvc_cvv_single_y() {
        let input = vec!["zuk".to_string(), "de'a".to_string()];
        let result = normalize(&input).unwrap();
        assert_eq!(result, vec!["zuk", "y", "de'a"]);
    }
}
