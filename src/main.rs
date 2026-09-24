use clap::{Arg, Command};
use smallvec::SmallVec;
use std::{
    collections::HashSet,
    fs::File,
    io::{self, BufRead, BufReader},
};
#[cfg(feature = "cli")]
use vlazba::cli_support::weights::{generate_weights, validate_words};
use vlazba::cli_support::{C, DEFAULT_WEIGHTS_STR, V, VERSION};
use vlazba::gismu::{GismuGenerator, GismuMatcher, GismuScorer};
use vlazba::jvozba::{
    jvokaha, jvozba,
    tools::{RafsiOptions, reconstruct_lujvo, search_selrafsi_from_rafsi2},
};

#[cfg(feature = "parallel")]
use rayon::prelude::*;

fn log(msg: &str) {
    eprintln!("{msg}");
}

fn default_rafsi(exp_rafsi: bool) -> RafsiOptions<'static> {
    RafsiOptions {
        exp_rafsi,
        custom_cmavo: None,
        custom_cmavo_exp: None,
        custom_gismu: None,
        custom_gismu_exp: None,
    }
}

fn main() -> anyhow::Result<()> {
    let matches = Command::new("Optimized Gismu Generator")
        .version(VERSION)
        .arg(Arg::new("words").help("Input words"))
        .arg(
            Arg::new("all-letters")
                .short('a')
                .long("all-letters")
                .help("Use all letters"),
        )
        .arg(
            Arg::new("shapes")
                .short('s')
                .long("shapes")
                .default_value("ccvcv,cvccv")
                .help("Shapes for gismu candidates"),
        )
        .arg(
            Arg::new("weights")
                .short('w')
                .long("weights")
                .default_value(DEFAULT_WEIGHTS_STR.as_str())
                .help("Weights for input words"),
        )
        .arg(
            Arg::new("deduplicate")
                .short('d')
                .long("deduplicate")
                .help("Path to gismu list for deduplication"),
        )
        .arg(
            Arg::new("jvozba")
                .long("jvozba")
                .help("Use jvozba function instead of gismu generation")
                .num_args(0)
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("forbid_la_lai_doi")
                .long("forbid-la-lai-doi")
                .help("Forbid la, lai, doi in lujvo")
                .num_args(0)
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("exp_rafsi")
                .long("exp-rafsi")
                .help("All experimental rafsi when generating lujvo")
                .num_args(0)
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("jvokaha")
                .long("jvokaha")
                .help("Use jvokaha function to split lujvo")
                .num_args(0)
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("reconstruct")
                .long("reconstruct")
                .help("Reconstruct a lujvo from its components")
                .num_args(0)
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("forbid_cmevla")
                .long("forbid-cmevla")
                .help("Forbid cmevla (name words) in lujvo reconstruction")
                .num_args(0)
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("best_only")
                .long("best-only")
                .help("With --jvozba: only compute score-optimal lujvo (DP; no full enumeration)")
                .num_args(0)
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("gimka")
                .long("gimka")
                .help("Find similar existing gismu using gimka function")
                .num_args(0)
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    if matches.get_flag("jvozba") {
        let words: Vec<String> = matches
            .get_one::<String>("words")
            .map(|s| s.split_whitespace().map(str::to_string).collect())
            .unwrap_or_default();

        let forbid_la_lai_doi = matches.get_flag("forbid_la_lai_doi");
        let exp_rafsi = matches.get_flag("exp_rafsi");
        let best_only = matches.get_flag("best_only");
        let results = jvozba(
            &words,
            forbid_la_lai_doi,
            false,
            best_only,
            &default_rafsi(exp_rafsi),
        );
        for result in results {
            log(&format!("{}: {}", result.lujvo, result.score));
        }
        return Ok(());
    }

    if matches.get_flag("reconstruct") {
        let lujvo: &str = matches
            .get_one::<String>("words")
            .map(String::as_str)
            .unwrap_or("");
        let exp_rafsi = matches.get_flag("exp_rafsi");
        let forbid_cmevla = matches.get_flag("forbid_cmevla");
        match reconstruct_lujvo(lujvo, forbid_cmevla, &default_rafsi(exp_rafsi)) {
            Ok(reconstructed) => {
                log(&format!("Reconstructed lujvo: {reconstructed}"));
            }
            Err(e) => {
                log(&format!("Error reconstructing lujvo: {e}"));
            }
        }
        return Ok(());
    }

    if matches.get_flag("gimka") {
        let candidate: &str = matches
            .get_one::<String>("words")
            .map(String::as_str)
            .unwrap_or("");
        let gismu_list_path = matches
            .get_one::<String>("deduplicate")
            .map(String::as_str)
            .unwrap_or("src/gismu-list.txt");

        log(&format!(
            "Looking for gismu similar to '{candidate}' using list: {gismu_list_path}"
        ));
        let gismus = read_gismu_list(gismu_list_path)?;
        let matcher = GismuMatcher::new(&gismus, None);
        let similar = matcher.gimka(candidate);

        if !similar.is_empty() {
            log("Similar gismu found:");
            for gismu in similar {
                log(&format!("- {gismu}"));
            }
        } else {
            log("No similar gismu found");
        }
        return Ok(());
    }

    if matches.get_flag("jvokaha") {
        let words: &str = matches
            .get_one::<String>("words")
            .map(String::as_str)
            .unwrap_or("");

        let results = jvokaha::jvokaha(words);
        let exp_rafsi = matches.get_flag("exp_rafsi");

        match results {
            Ok(result) => {
                let arr: Vec<String> = result
                    .into_iter()
                    .filter(|a| a.len() > 1)
                    .map(|rafsi| {
                        match search_selrafsi_from_rafsi2(&rafsi, &default_rafsi(exp_rafsi)) {
                            Some(selrafsi) => selrafsi,
                            None => format!("-{rafsi}-"),
                        }
                    })
                    .collect();
                log("Successfully decomposed lujvo:");
                for (index, rafsi) in arr.iter().enumerate() {
                    log(&format!("  {}: {}", index + 1, rafsi));
                }
            }
            Err(e) => {
                log(&format!("Error: {e}"));
            }
        }
        return Ok(());
    }

    let words: Vec<String> = matches
        .get_one::<String>("words")
        .map(|s| s.split_whitespace().map(str::to_string).collect())
        .unwrap_or_default();
    let all_letters = matches.contains_id("all-letters");
    let shapes: Vec<String> = matches
        .get_one::<String>("shapes")
        .unwrap()
        .split(',')
        .map(str::trim)
        .map(str::to_string)
        .collect();
    let weights = generate_weights(matches.get_one::<String>("weights").unwrap())?;

    let gismu_list_path = matches.get_one::<String>("deduplicate");

    validate_words(&words, &weights)?;

    let (c, v) = if all_letters {
        (
            C.chars().map(|s| s.to_string()).collect(),
            V.chars().map(|s| s.to_string()).collect(),
        )
    } else {
        letters_for_words(&words)
    };
    log(&format!(
        "Using letters {} and {}.",
        c.join(","),
        v.join(",")
    ));

    let candidate_iterator = GismuGenerator::new(c, v, shapes);
    let candidates: Vec<String> = candidate_iterator.iterator();
    log(&format!("{} candidates generated.", candidates.len()));

    let scorer = GismuScorer::new(&words, &weights);

    #[cfg(feature = "parallel")]
    let mut scores: Vec<(f32, &String, SmallVec<[f32; 6]>)> = candidates
        .par_iter()
        .map(|candidate| scorer.compute_score_with_name(candidate))
        .collect();
    #[cfg(not(feature = "parallel"))]
    let mut scores: Vec<(f32, &String, SmallVec<[f32; 6]>)> = candidates
        .iter()
        .map(|candidate| scorer.compute_score_with_name(candidate))
        .collect();

    scores.sort_unstable_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

    log("\n10 first gismu candidates are:\n");
    for record in scores.iter().take(10) {
        log(&format!("{record:?}"));
    }

    if let Some(gismu_list_path) = gismu_list_path {
        log("Reading list of gismu... ");
        let gismus = read_gismu_list(gismu_list_path)?;
        let matcher = GismuMatcher::new(&gismus, None);
        log("Excluding candidates similar to existing gismu...");
        if let Some(candidate) = deduplicate_candidates(&matcher, &scores) {
            log("The winner is....");
            log(&candidate.to_uppercase());
        } else {
            log("No suitable candidates found.");
        }
    }

    Ok(())
}

fn letters_for_words(words: &[String]) -> (Vec<String>, Vec<String>) {
    let word_set: HashSet<char> = words.iter().flat_map(|word| word.chars()).collect();

    (
        C.chars()
            .filter(|&c| word_set.contains(&c))
            .map(|s| s.to_string())
            .collect(),
        V.chars()
            .filter(|&c| word_set.contains(&c))
            .map(|s| s.to_string())
            .collect(),
    )
}

/// First score-sorted candidate that does **not** clash with an existing gismu.
fn deduplicate_candidates(
    matcher: &GismuMatcher<'_>,
    scores: &[(f32, &String, SmallVec<[f32; 6]>)],
) -> Option<String> {
    // Sequential: scores are sorted best-first; parallel any-order would pick randomly.
    scores.iter().find_map(
        |(_, candidate, _)| match matcher.find_similar_gismu(candidate) {
            Some(gismu) => {
                log(&format!(
                    "Candidate '{candidate}' too much like gismu '{gismu}'."
                ));
                None
            }
            None => Some((*candidate).clone()),
        },
    )
}

fn read_gismu_list(path: &str) -> io::Result<Vec<String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    reader.lines().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use smallvec::smallvec;

    #[test]
    fn deduplicate_picks_first_non_clashing() {
        let gismus = vec!["barda".to_string()];
        let matcher = GismuMatcher::new(&gismus, Some(4));
        let a = "bard".to_string();
        let b = "zzzz".to_string();
        let scores = [(2.0, &a, smallvec![1.0]), (1.0, &b, smallvec![0.0])];
        assert_eq!(
            deduplicate_candidates(&matcher, &scores).as_deref(),
            Some("zzzz")
        );
    }
}
