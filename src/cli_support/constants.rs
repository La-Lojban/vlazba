use std::collections::HashMap;
use std::sync::LazyLock;

pub const VERSION: &str = concat!("v", env!("CARGO_PKG_VERSION"));

pub static DEFAULT_WEIGHTS_STR: LazyLock<String> = LazyLock::new(|| {
    LANGUAGE_WEIGHTS
        .get("1985")
        .expect("1985 weights should exist")
        .iter()
        .map(|&weight| weight.to_string())
        .collect::<Vec<_>>()
        .join(",")
});

pub static LANGUAGE_WEIGHTS: LazyLock<HashMap<&'static str, Vec<f32>>> = LazyLock::new(|| {
    [
        ("1985", vec![0.36, 0.16, 0.21, 0.11, 0.09, 0.07]),
        ("1987", vec![0.36, 0.156, 0.208, 0.116, 0.087, 0.073]),
        ("1994", vec![0.348, 0.194, 0.163, 0.123, 0.088, 0.084]),
        ("1995", vec![0.347, 0.196, 0.16, 0.123, 0.089, 0.085]),
        ("1999", vec![0.334, 0.195, 0.187, 0.116, 0.081, 0.088]),
    ]
    .into_iter()
    .collect()
});

pub fn language_weights() -> &'static HashMap<&'static str, Vec<f32>> {
    &LANGUAGE_WEIGHTS
}

/// Lojban consonant inventory (compatibility name `C` re-exported from [`super`]).
pub const CONSONANTS: &str = "bcdfgjklmnprstvxz";
/// Lojban vowel inventory (compatibility name `V` re-exported from [`super`]).
pub const VOWELS: &str = "aeiou";
