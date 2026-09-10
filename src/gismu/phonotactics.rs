//! Phonotactic constants for gismu generation (CLL).

use std::collections::HashSet;
use std::sync::LazyLock;

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

pub(crate) static VALID_CC_INITIALS_SET: LazyLock<HashSet<&'static str>> =
    LazyLock::new(|| VALID_CC_INITIALS.iter().copied().collect());

pub(crate) static FORBIDDEN_CC_SET: LazyLock<HashSet<&'static str>> =
    LazyLock::new(|| FORBIDDEN_CC.iter().copied().collect());

pub(crate) static FORBIDDEN_CCC_SET: LazyLock<HashSet<&'static str>> =
    LazyLock::new(|| FORBIDDEN_CCC.iter().copied().collect());

pub(crate) static SIBILANT_SET: LazyLock<HashSet<char>> =
    LazyLock::new(|| SIBILANT.chars().collect());

pub(crate) static VOICED_SET: LazyLock<HashSet<char>> = LazyLock::new(|| VOICED.chars().collect());

pub(crate) static UNVOICED_SET: LazyLock<HashSet<char>> =
    LazyLock::new(|| UNVOICED.chars().collect());
