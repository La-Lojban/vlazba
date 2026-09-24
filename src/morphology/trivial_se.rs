use std::cell::RefCell;
use std::collections::HashSet;

use camxes_rs::camxes::LOJBAN_GRAMMAR;
use camxes_rs::camxes::peg::grammar::Peg;
use camxes_rs::camxes::peg::parsing::{ParseNode, ParseResult};

use super::cv::{CvInfo, cv_shape};
use super::lookup::{RafsiOptions, expand_lujvo_into_tanru, gismu_rafsi_list};

thread_local! {
    static PARSER: RefCell<Option<Peg>> = const { RefCell::new(None) };
}

/// A spelling-independent key for the expanded components of a brivla.
pub fn expansion_key(word: &str, options: &RafsiOptions<'_>) -> Option<String> {
    if is_brivla(word, options) {
        return serde_json::to_string(&vec![word]).ok();
    }
    let tanru = expand_lujvo_into_tanru(word, options).ok()?;
    let mut sources: Vec<&str> = tanru.split_whitespace().collect();
    // An outer KE is a wrapper rather than a component of the result.
    while sources.first().is_some_and(|s| *s == "ke") && sources.len() > 2 {
        sources.remove(0);
    }
    serde_json::to_string(&sources).ok()
}

pub fn trivial_se_base_expansion(
    word: &str,
    options: &RafsiOptions<'_>,
    se_words: Option<&HashSet<String>>,
) -> Option<String> {
    expansion_key(&trivial_se_base(word, options, se_words)?, options)
}

/// A common key for a base brivla, its spelling variants, and trivial SE forms.
pub fn trivial_expansion_key(
    word: &str,
    options: &RafsiOptions<'_>,
    se_words: Option<&HashSet<String>>,
) -> Option<String> {
    trivial_se_base_expansion(word, options, se_words).or_else(|| expansion_key(word, options))
}

/// Return the unconverted brivla when the expanded lujvo parses as exactly
/// one top-level tanru unit whose outermost operators are one or more SE.
pub fn trivial_se_base(
    word: &str,
    options: &RafsiOptions<'_>,
    se_words: Option<&HashSet<String>>,
) -> Option<String> {
    let tanru = expand_lujvo_into_tanru(word, options).ok()?;
    let sources: Vec<&str> = tanru.split_whitespace().collect();
    let mut start = parsed_outer_se_count(&sources, se_words)?;
    if start == 0 || !sources[..start].iter().all(|s| is_se(s, se_words)) {
        return None;
    }
    while sources.get(start).is_some_and(|s| *s == "ke") {
        start += 1;
    }
    let bare = &sources[start..];
    if bare.len() == 1 {
        return Some(bare[0].to_string());
    }
    let rafsi = super::camxes_segments::parsed_lujvo_rafsi(word)?;
    if rafsi.len() != sources.len() {
        return None;
    }
    let spelling = rafsi[start..].join("");
    if super::camxes_segments::parsed_lujvo_rafsi(&spelling).is_some() {
        return Some(spelling);
    }
    let source_words: Vec<String> = bare.iter().map(|s| (*s).to_string()).collect();
    super::compound::jvozba(&source_words, false, true, true, options)
        .first()
        .map(|candidate| candidate.lujvo.clone())
}

fn parsed_outer_se_count(sources: &[&str], se_words: Option<&HashSet<String>>) -> Option<usize> {
    // The full grammar knows the official SE spellings. Normalize imported SE
    // words only for syntax parsing; the stored expansion keeps their names.
    // The bundled grammar requires a word boundary after the final brivla.
    let phrase = sources
        .iter()
        .map(|s| if is_se(s, se_words) { "se" } else { *s })
        .collect::<Vec<_>>()
        .join(" ")
        + " ";
    PARSER.with(|slot| {
        let mut parser = slot.borrow_mut();
        if parser.is_none() {
            *parser = Peg::new(LOJBAN_GRAMMAR.0, LOJBAN_GRAMMAR.1).ok();
        }
        let ParseResult(_, consumed, _, parsed) = parser.as_ref()?.parse(&phrase);
        if consumed != phrase.len() {
            return Some(0);
        }
        let nodes = parsed.as_ref().as_ref().ok()?;
        let top = nodes
            .iter()
            .find_map(|n| find_node(n, "selbri_3", (0, phrase.len())))?;
        let ParseNode::NonTerminal { children, .. } = top else {
            return Some(0);
        };
        let units: Vec<&ParseNode> = children
            .iter()
            .filter(|n| node_name(n) == Some("selbri_4"))
            .collect();
        if units.len() != 1 {
            return Some(0);
        }
        let outer = find_node(units[0], "tanru_unit_2", (0, phrase.len()))?;
        Some(outer_se_count(outer))
    })
}

fn find_node<'a>(node: &'a ParseNode, name: &str, span: (usize, usize)) -> Option<&'a ParseNode> {
    let ParseNode::NonTerminal {
        name: node_name,
        span: node_span,
        children,
    } = node
    else {
        return None;
    };
    if node_name == name && (node_span.0, node_span.1) == span {
        return Some(node);
    }
    children
        .iter()
        .find_map(|child| find_node(child, name, span))
}

fn node_name(node: &ParseNode) -> Option<&str> {
    match node {
        ParseNode::NonTerminal { name, .. } => Some(name.as_str()),
        _ => None,
    }
}

fn outer_se_count(node: &ParseNode) -> usize {
    let ParseNode::NonTerminal { children, .. } = node else {
        return 0;
    };
    if !children.iter().any(|n| node_name(n) == Some("SE_clause")) {
        return 0;
    }
    1 + children
        .iter()
        .find(|n| node_name(n) == Some("tanru_unit_2"))
        .map_or(0, outer_se_count)
}

fn is_se(source: &str, se_words: Option<&HashSet<String>>) -> bool {
    se_words
        .filter(|words| !words.is_empty())
        .map_or(matches!(source, "se" | "te" | "ve" | "xe"), |words| {
            words.contains(source)
        })
}

fn is_brivla(source: &str, options: &RafsiOptions<'_>) -> bool {
    matches!(cv_shape(source), CvInfo::CVCCV | CvInfo::CCVCV)
        || gismu_rafsi_list(
            source,
            options.exp_rafsi,
            options.custom_gismu,
            options.custom_gismu_exp,
        )
        .is_some_and(|rafsi| !rafsi.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn options() -> RafsiOptions<'static> {
        RafsiOptions {
            exp_rafsi: true,
            custom_cmavo: None,
            custom_cmavo_exp: None,
            custom_gismu: None,
            custom_gismu_exp: None,
        }
    }

    #[test]
    fn trivial_and_nontrivial_se() {
        let options = options();
        assert_eq!(
            expand_lujvo_into_tanru("selkempa'isle", &options).unwrap(),
            "se ke prami selci"
        );
        assert_eq!(
            trivial_se_base("selpa'i", &options, None).as_deref(),
            Some("prami")
        );
        assert_eq!(
            trivial_se_base("selnunsle", &options, None).as_deref(),
            Some("nunsle")
        );
        assert_eq!(
            trivial_se_base("selkempa'isle", &options, None).as_deref(),
            Some("pa'isle")
        );
        assert_eq!(
            trivial_se_base("selselpa'i", &options, None).as_deref(),
            Some("prami")
        );
        assert_eq!(trivial_se_base("selpa'isle", &options, None), None);
        let key = trivial_expansion_key("pamsle", &options, None);
        assert_eq!(trivial_expansion_key("pa'isle", &options, None), key);
        assert_eq!(trivial_expansion_key("selkempa'isle", &options, None), key);
        assert_eq!(
            trivial_expansion_key("vlakra", &options, None),
            trivial_expansion_key("valkra", &options, None)
        );
        for (word, base) in [
            ("terpa'i", "prami"),
            ("velpa'i", "prami"),
            ("xelpa'i", "prami"),
        ] {
            assert_eq!(trivial_se_base(word, &options, None).as_deref(), Some(base));
        }
    }

    #[test]
    fn imported_se_list_controls_classification() {
        let words = HashSet::from(["te".to_owned()]);
        assert_eq!(
            trivial_se_base("terpa'i", &options(), Some(&words)).as_deref(),
            Some("prami")
        );
        assert_eq!(trivial_se_base("selpa'i", &options(), Some(&words)), None);
    }
}
