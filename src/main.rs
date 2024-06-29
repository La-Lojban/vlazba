use clap::{Arg, Command};
use jvozba::{jvokaha, tools::search_selrafsi_from_rafsi2};
use once_cell::sync::Lazy;
use rayon::prelude::*;
use regex::Regex;
use std::{
    collections::HashSet,
    fs::File,
    io::{self, BufRead, BufReader},
};

mod gismu_utils;
use gismu_utils::{language_weights, GismuGenerator, GismuMatcher, GismuScorer, C, V};
mod jvozba;

const VERSION: &str = "v0.7.3";

static DEFAULT_WEIGHTS_STR: Lazy<String> = Lazy::new(|| {
    language_weights()
        .get("1985")
        .expect("1985 weights should exist")
        .iter()
        .map(|&weight| weight.to_string())
        .collect::<Vec<_>>()
        .join(",")
});

fn log(msg: &str) {
    eprintln!("{}", msg);
}

fn generate_weights(weights_str: &str) -> anyhow::Result<Vec<f32>> {
    let re = Regex::new(r"(\d{4}|finprims)$")?;
    if re.is_match(weights_str) {
        language_weights()
            .get(weights_str)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("No weights registered for {}", weights_str))
    } else {
        weights_str
            .split(',')
            .map(|x| x.trim().parse::<f32>())
            .collect::<Result<Vec<f32>, _>>()
            .map_err(|_| anyhow::anyhow!("Values for weights must be numbers greater than zero"))
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
        .get_matches();

    if matches.get_flag("jvozba") {
        let words: Vec<String> = matches
            .get_one::<String>("words")
            .map(|s| s.split_whitespace().map(String::from).collect())
            .unwrap_or_default();

        let forbid_la_lai_doi = matches.get_flag("forbid_la_lai_doi");
        let exp_rafsi = matches.get_flag("exp_rafsi");
        let results = jvozba::jvozba(&words, forbid_la_lai_doi, exp_rafsi);
        for result in results {
            log(&format!("{}: {}", result.lujvo, result.score));
        }
        return Ok(());
    }

    if matches.get_flag("jvokaha") {
        let words: &str = matches
            .get_one::<String>("words")
            .map(String::as_str)
            .unwrap_or("");

        let results = jvokaha::jvokaha(words);

        match results {
            Ok(result) => {
                let exp_rafsi = matches.get_flag("exp_rafsi");
                let arr: Vec<String> = result
                    .into_iter()
                    .filter(|a| a.len() > 1)
                    .map(|rafsi| {
                        match search_selrafsi_from_rafsi2(&rafsi, exp_rafsi) {
                            Some(selrafsi) => selrafsi,
                            None => format!("-{}-", rafsi), // output as rafsi form; signify as unknown
                        }
                    })
                    .collect();
                log("Successfully decomposed lujvo:");
                for (index, rafsi) in arr.iter().enumerate() {
                    log(&format!("  {}: {}", index + 1, rafsi));
                }
            }
            Err(e) => {
                log(&format!("Error: {}", e));
            }
        }
        return Ok(());
    }

    let words: Vec<String> = matches
        .get_one::<String>("words")
        .map(|s| s.split_whitespace().map(String::from).collect())
        .unwrap_or_default();
    let all_letters = matches.contains_id("all-letters");
    let shapes: Vec<String> = matches
        .get_one::<String>("shapes")
        .unwrap()
        .split(',')
        .map(str::trim)
        .map(String::from)
        .collect();
    let weights = generate_weights(matches.get_one::<String>("weights").unwrap())?;

    let gismu_list_path = matches.get_one::<String>("deduplicate");

    validate_words(&words, &weights)?;

    let (c, v) = if all_letters {
        (
            C.chars().map(String::from).collect(),
            V.chars().map(String::from).collect(),
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

    let mut scores: Vec<(f32, &String, Vec<f32>)> = candidates
        .par_iter()
        .map(|candidate| scorer.compute_score_with_name(candidate))
        .collect();

    scores.sort_unstable_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

    log("\n10 first gismu candidates are:\n");
    for record in scores.iter().take(10) {
        log(&format!("{:?}", record));
    }

    if let Some(gismu_list_path) = gismu_list_path {
        log("Reading list of gismu... ");
        let gismus = read_gismu_list(gismu_list_path)?;
        let matcher = GismuMatcher::new(&gismus, None);
        log("Excluding candidates similar to existing gismu...");
        if let Some(candidate) = deduplicate_candidates(&matcher, &scores) {
            log("The winner is....");
            log(&candidate.to_uppercase().to_string());
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
            .map(String::from)
            .collect(),
        V.chars()
            .filter(|&c| word_set.contains(&c))
            .map(String::from)
            .collect(),
    )
}

fn deduplicate_candidates(
    matcher: &GismuMatcher,
    scores: &[(f32, &String, Vec<f32>)],
) -> Option<String> {
    scores.iter().find_map(|(_, candidate, _)| {
        matcher.find_similar_gismu(candidate).map(|gismu| {
            log(&format!(
                "Candidate '{}' too much like gismu '{}'.",
                candidate, gismu
            ));
            (*candidate).to_string()
        })
    })
}

fn validate_words(words: &[String], weights: &[f32]) -> anyhow::Result<()> {
    if words.len() != weights.len() {
        anyhow::bail!("Expected {} words as input", weights.len());
    }
    if words.iter().any(|word| word.len() < 2) {
        anyhow::bail!("Input words must be at least two letters long");
    }
    Ok(())
}

fn read_gismu_list(path: &str) -> io::Result<Vec<String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    reader.lines().collect()
}
