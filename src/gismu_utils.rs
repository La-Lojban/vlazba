use once_cell::sync::Lazy;
use rayon::prelude::*;
use smallvec::SmallVec;
use std::collections::HashSet;

use crate::libs::config::{
    FORBIDDEN_CC, FORBIDDEN_CCC, SIBILANT, SIMILARITIES, UNVOICED, VALID_CC_INITIALS, VOICED,
};

static VALID_CC_INITIALS_SET: Lazy<HashSet<&'static str>> =
    Lazy::new(|| VALID_CC_INITIALS.iter().cloned().collect());

static FORBIDDEN_CC_SET: Lazy<HashSet<&'static str>> =
    Lazy::new(|| FORBIDDEN_CC.iter().cloned().collect());

static FORBIDDEN_CCC_SET: Lazy<HashSet<&'static str>> =
    Lazy::new(|| FORBIDDEN_CCC.iter().cloned().collect());

static SIBILANT_SET: Lazy<HashSet<char>> = Lazy::new(|| SIBILANT.chars().collect());

static VOICED_SET: Lazy<HashSet<char>> = Lazy::new(|| VOICED.chars().collect());

static UNVOICED_SET: Lazy<HashSet<char>> = Lazy::new(|| UNVOICED.chars().collect());

fn lcs_length(a: &str, b: &str) -> f32 {
    let (a_bytes, b_bytes) = (a.as_bytes(), b.as_bytes());
    let (m, n) = (a_bytes.len(), b_bytes.len());
    
    // Ensure a is the shorter string to optimize space usage
    if m > n {
        return lcs_length(b, a);
    }
    
    // Use a single vector, initialized with zeros
    let mut current = vec![0; m + 1];

    for j in 1..=n {
        let mut prev = 0;
        for i in 1..=m {
            let temp = current[i];
            if a_bytes[i - 1] == b_bytes[j - 1] {
                current[i] = prev + 1;
            } else {
                current[i] = current[i].max(current[i - 1]);
            }
            prev = temp;
        }
    }

    current[m] as f32
}

pub struct GismuGenerator {
    c: Vec<String>,
    v: Vec<String>,
    shape_strings: Vec<String>,
}

impl GismuGenerator {
    pub fn new(c: Vec<String>, v: Vec<String>, shape_strings: Vec<String>) -> Self {
        Self {
            c,
            v,
            shape_strings,
        }
    }

    pub fn iterator(&self) -> Vec<String> {
        self.shape_strings
            .par_iter()
            .flat_map(|shape_string| self.shape_iterator(shape_string))
            .collect()
    }

    fn shape_iterator(&self, shape_string: &str) -> Vec<String> {
        let shape = self.shape_for_string(shape_string);
        let validator = self.shape_validator(shape_string);
    
        (0..shape.iter().map(|v| v.len()).product::<usize>())
            .into_par_iter()
            .filter_map(move |index| {
                let mut candidate = String::with_capacity(shape.len());
                let mut remaining = index;
                for choices in &shape {
                    let choice_index = remaining % choices.len();
                    remaining /= choices.len();
                    candidate.push_str(&choices[choice_index]);
                }
                if validator(&candidate) {
                    Some(candidate)
                } else {
                    None
                }
            })
            .collect()
    }

    fn shape_for_string(&self, string: &str) -> Vec<&[String]> {
        string
            .chars()
            .map(|c| match c.to_ascii_lowercase() {
                'c' => &self.c[..],
                'v' => &self.v[..],
                _ => &[],
            })
            .collect()
    }

    fn shape_validator(&self, shape: &str) -> impl Fn(&str) -> bool {
        type Predicate = Box<dyn Fn(&str) -> bool + Send + Sync>;

        let predicates: Vec<Predicate> = shape
            .chars()
            .zip(shape.chars().skip(1))
            .enumerate()
            .filter_map(|(i, (c1, c2))| {
                if c1.eq_ignore_ascii_case(&'c') && c2.eq_ignore_ascii_case(&'c') {
                    let mut p: Vec<Predicate> = vec![Box::new(self.validator_for_cc(i))];
                    if shape.chars().nth(i + 2) == Some('c') {
                        p.push(Box::new(self.validator_for_ccc(i)));
                    }
                    if i > 0 && shape[i..].starts_with("ccvcv") {
                        p.push(Box::new(self.invalidator_for_initial_cc(i)));
                    }
                    Some(p)
                } else {
                    None
                }
            })
            .flatten()
            .collect();

        move |x: &str| predicates.iter().all(|p| p(x))
    }

    fn validator_for_cc(&self, i: usize) -> impl Fn(&str) -> bool {
        move |x: &str| {
            if i == 0 {
                VALID_CC_INITIALS_SET.contains(&x[..2])
            } else {
                let j = i + 1;
                let c1 = x.chars().nth(i).unwrap();
                let c2 = x.chars().nth(j).unwrap();

                !(c1 == c2
                    || (VOICED_SET.contains(&c1) && UNVOICED_SET.contains(&c2))
                    || (UNVOICED_SET.contains(&c1) && VOICED_SET.contains(&c2))
                    || (SIBILANT_SET.contains(&c1) && SIBILANT_SET.contains(&c2))
                    || FORBIDDEN_CC_SET.contains(&x[i..=j]))
            }
        }
    }

    fn validator_for_ccc(&self, i: usize) -> impl Fn(&str) -> bool {
        move |x| !FORBIDDEN_CCC_SET.contains(&x[i..=i + 2])
    }

    fn invalidator_for_initial_cc(&self, i: usize) -> impl Fn(&str) -> bool {
        move |x| !VALID_CC_INITIALS_SET.contains(&x[i..=i + 1])
    }
}

pub struct GismuScorer<'a> {
    input_words: &'a [String],
    weights: SmallVec<[f32; 6]>,
}

impl<'a> GismuScorer<'a> {
    pub fn new(input_words: &'a [String], weights: &[f32]) -> Self {
        Self {
            input_words,
            weights: SmallVec::from_slice(weights),
        }
    }

    fn compute_score(&self, candidate: &str) -> (f32, SmallVec<[f32; 6]>) {
        let similarity_scores: SmallVec<[f32; 6]> = self
            .input_words
            .iter()
            .map(|word| {
                let lcs_len = lcs_length(candidate, word);
                let score = match lcs_len {
                    0.0 | 1.0 => 0.0,
                    2.0 => self.score_dyad_by_pattern(candidate, word),
                    _ => lcs_len,
                };
                score / word.len() as f32
            })
            .collect();

        let weighted_sum = self.calculate_weighted_sum(&similarity_scores);
        (weighted_sum, similarity_scores)
    }

    pub fn compute_score_with_name<'b>(
        &self,
        candidate: &'b String,
    ) -> (f32, &'b String, SmallVec<[f32; 6]>) {
        let (weighted_sum, similarity_scores) = self.compute_score(candidate);
        (weighted_sum, candidate, similarity_scores)
    }

    fn score_dyad_by_pattern(&self, candidate: &str, input_word: &str) -> f32 {
        let l = candidate.len();
        let iw02: String = input_word.chars().step_by(2).collect();
        let iw12: String = input_word.chars().skip(1).step_by(2).collect();

        let mut score = 0.0;

        for i in 0..(l - 2) {
            let dyad = &candidate[i..(i + 2)];
            if iw02.contains(dyad) || iw12.contains(dyad) {
                score = 2.0;
                break;
            }
        }

        if score == 0.0 {
            for i in 0..(l - 1) {
                let dyad = &candidate[i..(i + 2)];
                if input_word.contains(dyad) {
                    score = 2.0;
                    break;
                }
            }
        }

        score
    }

    fn calculate_weighted_sum(&self, scores: &SmallVec<[f32; 6]>) -> f32 {
        scores
            .iter()
            .zip(self.weights.iter())
            .map(|(&score, &weight)| score * weight)
            .sum()
    }
}

pub struct GismuMatcher<'a> {
    gismus: &'a [String],
    stem_length: usize,
}

impl<'a> GismuMatcher<'a> {
    pub fn new(gismus: &'a [String], stem_length: Option<usize>) -> Self {
        Self {
            gismus,
            stem_length: stem_length.unwrap_or(4),
        }
    }

    pub fn find_similar_gismu(&self, candidate: &str) -> Option<String> {
        let candidate = candidate.trim_end();

        self.gismus
            .iter()
            .find(|word| self.match_gismu(word, candidate))
            .cloned()
    }

    fn match_gismu(&self, gismu: &str, candidate: &str) -> bool {
        self.match_stem(gismu, candidate) || self.match_structure(gismu, candidate)
    }

    fn match_structure(&self, gismu: &str, candidate: &str) -> bool {
        let common_len = candidate.len().min(gismu.len());
        (0..common_len).any(|i| {
            self.strings_match_except(gismu, candidate, i, common_len)
                && self
                    .match_structural_pattern(&gismu[i..i + 1], candidate.chars().nth(i).unwrap())
        })
    }

    fn match_structural_pattern(&self, letter: &str, c: char) -> bool {
        SIMILARITIES
            .iter()
            .find(|&&(key, _)| key == c.to_ascii_lowercase()).is_some_and(|&(_, pattern)| {
                pattern.contains(letter) || pattern.is_empty()
            })
    }

    fn match_stem(&self, gismu: &str, candidate: &str) -> bool {
        candidate.len() >= self.stem_length && gismu.starts_with(&candidate[..self.stem_length])
    }

    fn strings_match_except(&self, x: &str, y: &str, i: usize, j: usize) -> bool {
        x[..i] == y[..i] && x[(i + 1)..j] == y[(i + 1)..j]
    }
}
