//! Candidate generation over CV shapes.

#[cfg(feature = "parallel")]
use rayon::prelude::*;

use super::phonotactics::{
    FORBIDDEN_CCC_SET, FORBIDDEN_CC_SET, SIBILANT_SET, UNVOICED_SET, VALID_CC_INITIALS_SET,
    VOICED_SET,
};

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

    /// Generate all valid candidates for the configured shapes.
    pub fn generate(&self) -> Vec<String> {
        self.iterator()
    }

    /// Compatibility alias for [`Self::generate`].
    pub fn iterator(&self) -> Vec<String> {
        #[cfg(feature = "parallel")]
        {
            self.shape_strings
                .par_iter()
                .flat_map(|shape_string| self.shape_iterator(shape_string))
                .collect()
        }
        #[cfg(not(feature = "parallel"))]
        {
            self.shape_strings
                .iter()
                .flat_map(|shape_string| self.shape_iterator(shape_string))
                .collect()
        }
    }

    fn shape_iterator(&self, shape_string: &str) -> Vec<String> {
        let shape = self.shape_for_string(shape_string);
        let validator = self.shape_validator(shape_string);
        let total = shape.iter().map(|v| v.len()).product::<usize>();

        #[cfg(feature = "parallel")]
        let iter = (0..total).into_par_iter();
        #[cfg(not(feature = "parallel"))]
        let iter = 0..total;

        iter.filter_map(move |index| {
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

    fn shape_validator(&self, shape: &str) -> impl Fn(&str) -> bool + Send + Sync {
        type Predicate = Box<dyn Fn(&str) -> bool + Send + Sync>;

        let predicates: Vec<Predicate> = shape
            .chars()
            .zip(shape.chars().skip(1))
            .enumerate()
            .filter_map(|(i, (c1, c2))| {
                if c1.eq_ignore_ascii_case(&'c') && c2.eq_ignore_ascii_case(&'c') {
                    let mut p: Vec<Predicate> = vec![Box::new(Self::validator_for_cc(i))];
                    if shape.chars().nth(i + 2) == Some('c') {
                        p.push(Box::new(Self::validator_for_ccc(i)));
                    }
                    if i > 0 && shape[i..].starts_with("ccvcv") {
                        p.push(Box::new(Self::invalidator_for_initial_cc(i)));
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

    fn validator_for_cc(i: usize) -> impl Fn(&str) -> bool + Send + Sync {
        move |x: &str| {
            if i == 0 {
                VALID_CC_INITIALS_SET.contains(&x[..2])
            } else {
                let j = i + 1;
                let c1 = x.as_bytes()[i] as char;
                let c2 = x.as_bytes()[j] as char;

                !(c1 == c2
                    || (VOICED_SET.contains(&c1) && UNVOICED_SET.contains(&c2))
                    || (UNVOICED_SET.contains(&c1) && VOICED_SET.contains(&c2))
                    || (SIBILANT_SET.contains(&c1) && SIBILANT_SET.contains(&c2))
                    || FORBIDDEN_CC_SET.contains(&x[i..=j]))
            }
        }
    }

    fn validator_for_ccc(i: usize) -> impl Fn(&str) -> bool + Send + Sync {
        move |x| !FORBIDDEN_CCC_SET.contains(&x[i..=i + 2])
    }

    fn invalidator_for_initial_cc(i: usize) -> impl Fn(&str) -> bool + Send + Sync {
        move |x| !VALID_CC_INITIALS_SET.contains(&x[i..=i + 1])
    }
}

