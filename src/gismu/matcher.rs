//! Gismu clash / similarity matching (gimka).

use super::phonotactics::SIMILARITIES;

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

    /// Find all gismu similar to the candidate word
    pub fn gimka(&self, candidate: &str) -> Vec<String> {
        let candidate = candidate.trim_end();
        self.gismus
            .iter()
            .filter(|word| self.match_gismu(word, candidate))
            .cloned()
            .collect()
    }

    fn match_gismu(&self, gismu: &str, candidate: &str) -> bool {
        self.match_stem(gismu, candidate) || self.match_structure(gismu, candidate)
    }

    fn match_structure(&self, gismu: &str, candidate: &str) -> bool {
        let common_len = candidate.len().min(gismu.len());
        (0..common_len).any(|i| {
            self.strings_match_except(gismu, candidate, i, common_len)
                && self.match_structural_pattern(
                    &gismu[i..i + 1],
                    candidate.as_bytes().get(i).copied().unwrap_or(0) as char,
                )
        })
    }

    fn match_structural_pattern(&self, letter: &str, c: char) -> bool {
        SIMILARITIES
            .iter()
            .find(|&&(key, _)| key == c.to_ascii_lowercase())
            .is_some_and(|&(_, pattern)| pattern.contains(letter) || pattern.is_empty())
    }

    fn match_stem(&self, gismu: &str, candidate: &str) -> bool {
        candidate.len() >= self.stem_length && gismu.starts_with(&candidate[..self.stem_length])
    }

    fn strings_match_except(&self, x: &str, y: &str, i: usize, j: usize) -> bool {
        x[..i] == y[..i] && x[(i + 1)..j] == y[(i + 1)..j]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dedup_polarity_skips_clashes() {
        // Document intended polarity: find_similar means clash → reject candidate.
        let gismus = vec!["barda".to_string()];
        let matcher = GismuMatcher::new(&gismus, Some(4));
        assert!(matcher.find_similar_gismu("bard").is_some());
        assert!(matcher.find_similar_gismu("zzzz").is_none());
    }
}
