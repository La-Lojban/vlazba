use clap::{Arg, Command};
use jvozba::{jvokaha, tools::search_selrafsi_from_rafsi2, jvozba};
use rayon::prelude::*;
use std::{
    collections::HashSet,
    fs::File,
    io::{self, BufRead, BufReader},
    sync::Arc,
};
use smallvec::SmallVec;

mod libs;
use libs::{cli::{generate_weights, validate_words}, config::{C, DEFAULT_WEIGHTS_STR, V, VERSION}};

mod gismu_utils;
use gismu_utils::{GismuGenerator, GismuMatcher, GismuScorer};
mod jvozba;

fn log(msg: &str) {
    eprintln!("{}", msg);
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
        .get_matches();

    if matches.get_flag("jvozba") {
        let words: Vec<String> = matches
            .get_one::<String>("words")
            .map(|s| s.split_whitespace().map(|word| word.to_string()).collect())
            .unwrap_or_default();

        let forbid_la_lai_doi = matches.get_flag("forbid_la_lai_doi");
        let exp_rafsi = matches.get_flag("exp_rafsi");
        let results = jvozba(&words, forbid_la_lai_doi, exp_rafsi, false);
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
        match jvozba::tools::reconstruct_lujvo(lujvo, exp_rafsi, forbid_cmevla) {
            Ok(reconstructed) => {
                log(&format!("Reconstructed lujvo: {}", reconstructed));
            }
            Err(e) => {
                log(&format!("Error reconstructing lujvo: {}", e));
            }
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
        .map(|s| s.split_whitespace().map(|word| word.to_string()).collect())
        .unwrap_or_default();
    let all_letters = matches.contains_id("all-letters");
    let shapes: Vec<String> = matches
        .get_one::<String>("shapes")
        .unwrap()
        .split(',')
        .map(str::trim)
        .map(|s| s.to_string())
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

    let mut scores: Vec<(f32, &String, SmallVec<[f32; 6]>)> = candidates
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
        let matcher = Arc::new(GismuMatcher::new(&gismus, None));
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
            .map(|s| s.to_string())
            .collect(),
        V.chars()
            .filter(|&c| word_set.contains(&c))
            .map(|s| s.to_string())
            .collect(),
    )
}

fn deduplicate_candidates(
    matcher: &Arc<GismuMatcher>,
    scores: &[(f32, &String, SmallVec<[f32; 6]>)],
) -> Option<String> {
    scores.par_iter().find_map_any(|(_, candidate, _)| {
        matcher.find_similar_gismu(candidate).map(|gismu| {
            log(&format!(
                "Candidate '{}' too much like gismu '{}'.",
                candidate, gismu
            ));
            (*candidate).to_string()
        })
    })
}

fn read_gismu_list(path: &str) -> io::Result<Vec<String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    reader.lines().collect()
}
