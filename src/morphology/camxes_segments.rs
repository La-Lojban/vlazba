use std::collections::HashSet;
use camxes_rs::camxes::peg::parsing::ParseNode;
use camxes_rs::camxes::peg::{grammar::Peg, parsing::ParseResult};
use crate::morphology::decompose::jvokaha;
use crate::morphology::lookup::{rafsi_candidates, resolve_selrafsi, RafsiOptions};
use crate::morphology::score::lujvo_score;
use crate::morphology::compound::normalize;

/// Collects lujvo rafsi segment strings from the camxes parse tree.
/// Expands fu'ivla and stressed_*_rafsi into
/// "rafsi + 'y" / "rafsi + y" for readable decomposition.
pub fn lujvo_segments_from_nodes(input: &str, nodes: &[ParseNode]) -> Option<Vec<String>> {
    fn find_lujvo_core(nodes: &[ParseNode]) -> Option<&ParseNode> {
        for node in nodes {
            if let ParseNode::NonTerminal { name, children, .. } = node {
                if name == "lujvo_core" {
                    return Some(node);
                }
                if let Some(found) = find_lujvo_core(children) {
                    return Some(found);
                }
            }
        }
        None
    }

    let lujvo_core = find_lujvo_core(nodes)?;
    let ParseNode::NonTerminal { children, .. } = lujvo_core else {
        return None;
    };

    let mut parts: Vec<(usize, String)> = Vec::new();

    for node in children {
        if let ParseNode::NonTerminal {
            name,
            span,
            children: sub,
        } = node
        {
            let (s, e) = (span.0, span.1);
            if s >= e || e > input.len() {
                continue;
            }
            let text = input[s..e].to_string();
            if name == "stressed_fuhivla_rafsi" || name == "fuhivla_rafsi" {
                let mut rafsi_end = s;
                let mut hy_start = e;
                for n in sub {
                    if let ParseNode::NonTerminal {
                        name: nname,
                        span: nspan,
                        ..
                    } = n
                    {
                        let (ns, ne) = (nspan.0, nspan.1);
                        if nname == "fuhivla_trim" || nname == "fuhivla_head" {
                            rafsi_end = ne;
                        }
                        if nname == "onset" && ne <= e {
                            rafsi_end = rafsi_end.max(ne);
                        }
                        if (nname == "h" || nname == "y") && ns < hy_start {
                            hy_start = ns;
                        }
                    }
                }
                if rafsi_end > s {
                    parts.push((s, input[s..rafsi_end].to_string()));
                }
                if hy_start < e {
                    parts.push((hy_start, input[hy_start..e].to_string()));
                }
            } else if matches!(name.as_str(), "stressed_hy_rafsi" | "stressed_y_rafsi" | "hy_rafsi" | "y_rafsi") {
                let mut rafsi_end = s;
                let mut hy_start = e;
                for n in sub {
                    if let ParseNode::NonTerminal {
                        name: nname,
                        span: nspan,
                        ..
                    } = n
                    {
                        let (ns, ne) = (nspan.0, nspan.1);
                        if nname != "h" && nname != "y" && ne > rafsi_end {
                            rafsi_end = ne;
                        }
                        if (nname == "h" || nname == "y") && ns < hy_start {
                            hy_start = ns;
                        }
                    }
                }
                if rafsi_end > s {
                    parts.push((s, input[s..rafsi_end].to_string()));
                }
                if hy_start < e {
                    parts.push((hy_start, input[hy_start..e].to_string()));
                }
            } else {
                parts.push((s, text));
            }
        }
    }

    parts.sort_by_key(|(start, _)| *start);
    Some(parts.into_iter().map(|(_, s)| s).collect())
}

/// Reconstruct a lujvo containing fu'ivla rafsi from its ordered source words.
/// Score combinations of known source-word rafsi, accepting only spellings
/// that camxes parses back into the same ordered components.
pub fn reconstruct_fuhivla_lujvo(
    word: &str,
    source_words: &[String],
    parser: &Peg,
    options: &RafsiOptions<'_>,
) -> Option<String> {
    let fuhivla: Vec<bool> = source_words.iter().map(|source| {
        let ParseResult(_, _, _, parsed) = parser.parse(source);
        parsed.as_ref().as_ref().is_ok_and(|nodes| contains_fuhivla(nodes))
    }).collect();
    reconstruct_with_sources(word, source_words, &fuhivla, parser, options)
}

fn reconstruct_with_sources(
    word: &str,
    source_words: &[String],
    fuhivla: &[bool],
    parser: &Peg,
    options: &RafsiOptions<'_>,
) -> Option<String> {
    if jvokaha(word).is_ok() {
        return None;
    }
    let ParseResult(_, _, _, parsed) = parser.parse(word);
    let parts = lujvo_segments_from_nodes(word, parsed.as_ref().as_ref().ok()?)?;
    let mut original_rafsi = Vec::new();
    let mut original_glue = Vec::new();
    for part in parts {
        if matches!(part.as_str(), "y" | "'y" | "r" | "n") {
            if original_rafsi.is_empty() { return None; }
            if original_glue.len() < original_rafsi.len() {
                original_glue.push(String::new());
            }
            original_glue.last_mut().map(|g: &mut String| g.push_str(&part))?;
        } else {
            if !original_rafsi.is_empty() && original_glue.len() < original_rafsi.len() {
                original_glue.push(String::new());
            }
            original_rafsi.push(part);
        }
    }
    if original_rafsi.len() < 2 || original_rafsi.len() != source_words.len()
        || original_glue.len() + 1 != original_rafsi.len() {
        return None;
    }

    if !fuhivla.iter().any(|&is_fuhivla| is_fuhivla) {
        return None;
    }

    let choices: Vec<Vec<String>> = source_words.iter().enumerate().map(|(i, source)| {
        let mut candidates = if fuhivla[i] {
            vec![if i + 1 == source_words.len() { source.clone() } else { original_rafsi[i].clone() }]
        } else {
            rafsi_candidates(source, i + 1 == source_words.len(), options)
        };
        if !candidates.contains(&original_rafsi[i]) {
            candidates.push(original_rafsi[i].clone());
        }
        candidates.sort();
        candidates.dedup();
        candidates
    }).collect();

    let mut choice_indices = vec![0; choices.len()];
    let mut seen = HashSet::new();
    let mut best: Option<(i32, String)> = None;
    const MAX_CANDIDATES: usize = 100_000;
    loop {
        let selected: Vec<String> = choices.iter().enumerate().map(|(i, c)| c[choice_indices[i]].clone()).collect();
        let mut glue_choices: Vec<Vec<String>> = original_glue.iter().map(|g| vec![g.clone()]).collect();
        if let Ok(normalized) = normalize(&selected) {
            let normalized = normalized.join("");
            let mut cursor = 0;
            for (i, rafsi) in selected.iter().enumerate() {
                if let Some(offset) = normalized[cursor..].find(rafsi) {
                    if i > 0 {
                        let glue = normalized[cursor..cursor + offset].to_string();
                        if !glue_choices[i - 1].contains(&glue) {
                            glue_choices[i - 1].push(glue);
                        }
                    }
                    cursor += offset + rafsi.len();
                }
            }
        }
        for (i, is_fuhivla) in fuhivla.iter().enumerate().take(fuhivla.len() - 1) {
            if *is_fuhivla {
                // Camxes can reinterpret a final i/u as the onset glide of y.
                // Segment text alone does not detect that syllable change:
                // preserve the vowel-ending rafsi with an explicit h+y boundary.
                if selected[i].ends_with(['a', 'e', 'i', 'o', 'u']) {
                    glue_choices[i] = vec!["'y".into()];
                } else if !glue_choices[i].iter().any(|g| g == "y") {
                    glue_choices[i].push("y".into());
                }
            }
        }
        let mut glue_indices = vec![0; glue_choices.len()];
        loop {
            let mut segments = vec![selected[0].clone()];
            for i in 0..glue_choices.len() {
                let glue = &glue_choices[i][glue_indices[i]];
                if !glue.is_empty() {
                    segments.push(glue.clone());
                }
                segments.push(selected[i + 1].clone());
            }
            let candidate = segments.join("");
            if seen.insert(candidate.clone()) {
                if seen.len() > MAX_CANDIDATES { return None; }
                let score = lujvo_score(&segments);
                if best.as_ref().is_none_or(|current| (score, &candidate) < (current.0, &current.1)) {
                    let ParseResult(_, _, _, parsed) = parser.parse(&candidate);
                    if parsed.as_ref().as_ref().ok()
                        .and_then(|nodes| lujvo_segments_from_nodes(&candidate, nodes))
                        .is_some_and(|parsed_parts| parsed_parts == segments) {
                        best = Some((score, candidate));
                    }
                }
            }
            let Some(i) = (0..glue_indices.len()).rev().find(|&i| glue_indices[i] + 1 < glue_choices[i].len()) else { break; };
            glue_indices[i] += 1;
            for index in glue_indices.iter_mut().skip(i + 1) { *index = 0; }
        }
        let Some(i) = (0..choice_indices.len()).rev().find(|&i| choice_indices[i] + 1 < choices[i].len()) else { break; };
        choice_indices[i] += 1;
        for index in choice_indices.iter_mut().skip(i + 1) { *index = 0; }
    }
    best.map(|(_, word)| word)
}

fn contains_fuhivla(nodes: &[ParseNode]) -> bool {
    nodes.iter().any(|node| match node {
        ParseNode::NonTerminal { name, children, .. } =>
            name == "fuhivla" || contains_fuhivla(children),
        _ => false,
    })
}

/// Infer known source words and preserve any fu'ivla rafsi in every position.
pub(crate) fn reconstruct_with_inferred_sources(word: &str, options: &RafsiOptions<'_>) -> Option<String> {
    let parser = Peg::new("text", include_str!("lojban.peg")).ok()?;
    let ParseResult(_, _, _, parsed) = parser.parse(word);
    let segments = lujvo_segments_from_nodes(word, parsed.as_ref().as_ref().ok()?)?;
    let rafsi: Vec<_> = segments.iter()
        .filter(|part| !matches!(part.as_str(), "y" | "'y" | "r" | "n"))
        .collect();
    if rafsi.len() < 2 { return None; }
    let mut sources = Vec::with_capacity(rafsi.len());
    let mut fuhivla = Vec::with_capacity(rafsi.len());
    for part in rafsi {
        if let Some(source) = resolve_selrafsi(part, options) {
            sources.push(source);
            fuhivla.push(false);
        } else {
            sources.push(part.to_owned());
            fuhivla.push(true);
        }
    }
    reconstruct_with_sources(word, &sources, &fuhivla, &parser, options)
}

#[cfg(test)]
mod tests {
    use super::*;
    use camxes_rs::camxes::peg::parsing::Span;

    #[test]
    fn preserves_vowel_fuhivla_hyphen() {
        let parser = Peg::new("text", include_str!("lojban.peg")).unwrap();
        let options = RafsiOptions { exp_rafsi: true, custom_cmavo: None, custom_cmavo_exp: None, custom_gismu: None, custom_gismu_exp: None };
        for (word, sources, expected) in [
            ("krakatau'yvalsi", vec!["krakatau", "valsi"], "krakatau'yvla"),
            ("krakatau'yvla", vec!["krakatau", "valsi"], "krakatau'yvla"),
            ("krakatu'yvalsi", vec!["krakatu", "valsi"], "krakatu'yvla"),
            ("krakatai'yvalsi", vec!["krakatai", "valsi"], "krakatai'yvla"),
            ("klamykrakatau'yvalsi", vec!["klama", "krakatau", "valsi"], "klamykrakatau'yvla"),
            ("krakatau'ykrakatau'yvalsi", vec!["krakatau", "krakatau", "valsi"], "krakatau'ykrakatau'yvla"),
        ] {
            let sources: Vec<String> = sources.into_iter().map(String::from).collect();
            assert_eq!(reconstruct_fuhivla_lujvo(word, &sources, &parser, &options).as_deref(), Some(expected), "{word}");
            assert_eq!(crate::morphology::lookup::reconstruct_lujvo(word, true, &options).unwrap(), expected, "{word}");
        }
    }

    #[test]
    fn reconstruct_inferred_fuhivla_with_shorter_rafsi() {
        let options = RafsiOptions { exp_rafsi: true, custom_cmavo: None, custom_cmavo_exp: None, custom_gismu: None, custom_gismu_exp: None };
        assert_eq!(reconstruct_with_inferred_sources("valsykrakatu", &options).as_deref(), Some("valykrakatu"));
        assert_eq!(reconstruct_with_inferred_sources("valykrakatu", &options).as_deref(), Some("valykrakatu"));
        assert!(lujvo_score(&["val".into(), "y".into(), "krakatu".into()])
            < lujvo_score(&["vals".into(), "y".into(), "krakatu".into()]));
    }

    fn node(name: &str, start: usize, end: usize, children: Vec<ParseNode>) -> ParseNode {
        ParseNode::NonTerminal {
            name: name.into(),
            span: Span(start, end),
            children,
        }
    }

    #[test]
    fn splits_fuhivla_rafsi_from_hyphen() {
        let nodes = vec![node(
            "lujvo_core",
            0,
            11,
            vec![
                node(
                    "fuhivla_rafsi",
                    0,
                    7,
                    vec![
                        node("fuhivla_head", 0, 5, vec![]),
                        node("onset", 5, 6, vec![]),
                        node("y", 6, 7, vec![]),
                    ],
                ),
                node("short_final_rafsi", 7, 11, vec![]),
            ],
        )];
        assert_eq!(
            lujvo_segments_from_nodes("tci'ilyfi'e", &nodes),
            Some(vec!["tci'il".into(), "y".into(), "fi'e".into()])
        );
    }

    #[test]
    fn reconstructs_fuhivla_rafsi_in_multiple_positions() {
        let grammar = include_str!("lojban.peg");
        let parser = Peg::new("text", grammar).expect("Lojban parser");
        let words = vec!["tci'ile".into(), "finpe".into()];
        let options = RafsiOptions {
            exp_rafsi: true,
            custom_cmavo: None,
            custom_cmavo_exp: None,
            custom_gismu: None,
            custom_gismu_exp: None,
        };
        assert_eq!(
            reconstruct_fuhivla_lujvo("tci'ilyfinpe", &words, &parser, &options),
            Some("tci'ilyfi'e".into())
        );
        assert_eq!(
            reconstruct_fuhivla_lujvo("tci'ilyfi'e", &words, &parser, &options),
            Some("tci'ilyfi'e".into())
        );
        for (word, sources, expected) in [
            ("klamytci'ilyfinpe", vec!["klama", "tci'ile", "finpe"], "klamytci'ilyfi'e"),
            ("tci'ilyfinpyklama", vec!["tci'ile", "finpe", "klama"], "tci'ilyfipkla"),
            ("tci'ilytci'ilyfinpe", vec!["tci'ile", "tci'ile", "finpe"], "tci'ilytci'ilyfi'e"),
        ] {
            let sources: Vec<String> = sources.into_iter().map(String::from).collect();
            assert_eq!(
                reconstruct_fuhivla_lujvo(word, &sources, &parser, &options),
                Some(expected.into()),
                "{word}"
            );
        }
    }
}
