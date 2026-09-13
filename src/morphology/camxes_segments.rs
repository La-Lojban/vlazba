use camxes_rs::camxes::peg::parsing::ParseNode;

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
            } else if name == "stressed_hy_rafsi" || name == "stressed_y_rafsi" {
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

#[cfg(test)]
mod tests {
    use super::*;
    use camxes_rs::camxes::peg::parsing::Span;

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
}
