//! Weighted LCS-based scoring of gismu candidates against source words.

use smallvec::SmallVec;

fn lcs_length(a: &str, b: &str) -> f32 {
    let (a_bytes, b_bytes) = (a.as_bytes(), b.as_bytes());
    let (m, n) = (a_bytes.len(), b_bytes.len());

    if m > n {
        return lcs_length(b, a);
    }

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

