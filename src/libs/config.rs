use once_cell::sync::Lazy;
use std::collections::HashMap;

pub const VERSION: &str = "v0.9.0";

pub static DEFAULT_WEIGHTS_STR: Lazy<String> = Lazy::new(|| {
    language_weights()
        .get("1985")
        .expect("1985 weights should exist")
        .iter()
        .map(|&weight| weight.to_string())
        .collect::<Vec<_>>()
        .join(",")
});

pub fn language_weights() -> HashMap<&'static str, Vec<f32>> {
    [
        ("1985", vec![0.36, 0.16, 0.21, 0.11, 0.09, 0.07]),
        ("1987", vec![0.36, 0.156, 0.208, 0.116, 0.087, 0.073]),
        ("1994", vec![0.348, 0.194, 0.163, 0.123, 0.088, 0.084]),
        ("1995", vec![0.347, 0.196, 0.16, 0.123, 0.089, 0.085]),
        ("1999", vec![0.334, 0.195, 0.187, 0.116, 0.081, 0.088]),
    ]
    .iter()
    .cloned()
    .collect()
}

pub const C: &str = "bcdfgjklmnprstvxz";
pub const V: &str = "aeiou";

pub const VALID_CC_INITIALS: &[&str] = &[
    "bl", "br", "cf", "ck", "cl", "cm", "cn", "cp", "cr", "ct", "dj", "dr", "dz", "fl", "fr", "gl",
    "gr", "jb", "jd", "jg", "jm", "jv", "kl", "kr", "ml", "mr", "pl", "pr", "sf", "sk", "sl", "sm",
    "sn", "sp", "sr", "st", "tc", "tr", "ts", "vl", "vr", "xl", "xr", "zb", "zd", "zg", "zm", "zv",
];

pub const FORBIDDEN_CC: &[&str] = &["cx", "kx", "xc", "xk", "mz"];

pub const FORBIDDEN_CCC: &[&str] = &["ndj", "ndz", "ntc", "nts"];

pub const SIBILANT: &str = "cjsz";
pub const VOICED: &str = "bdgjvz";
pub const UNVOICED: &str = "cfkpstx";


pub static SIMILARITIES: [(char, &str); 17] = [
    ('b', "pv"),
    ('c', "js"),
    ('d', "t"),
    ('f', "pv"),
    ('g', "kx"),
    ('j', "cz"),
    ('k', "gx"),
    ('l', "r"),
    ('m', "n"),
    ('n', "m"),
    ('p', "bf"),
    ('r', "l"),
    ('s', "cz"),
    ('t', "d"),
    ('v', "bf"),
    ('x', "gk"),
    ('z', "js"),
];