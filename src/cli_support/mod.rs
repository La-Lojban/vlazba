//! Shared helpers for the `vlazba` binary (weights, version, letter inventories).

#[cfg(feature = "cli")]
pub mod weights;

pub mod constants;

pub use constants::{
    CONSONANTS as C, DEFAULT_WEIGHTS_STR, LANGUAGE_WEIGHTS, VERSION, VOWELS as V, language_weights,
};
