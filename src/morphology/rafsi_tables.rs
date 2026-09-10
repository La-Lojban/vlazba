use std::collections::HashMap;
use std::sync::LazyLock;

type RafsiMap = HashMap<String, Vec<String>>;
type ReverseMap = HashMap<String, String>;

static GISMU_RAFSI_LIST: LazyLock<RafsiMap> = LazyLock::new(|| {
    serde_json::from_str(include_str!("gismu_rafsi_list.json"))
        .expect("Failed to parse gismu_rafsi_list.json")
});

static GISMU_RAFSI_LIST_EXP: LazyLock<RafsiMap> = LazyLock::new(|| {
    serde_json::from_str(include_str!("gismu_rafsi_list_exp.json"))
        .expect("Failed to parse gismu_rafsi_list_exp.json")
});

static CMAVO_RAFSI_LIST: LazyLock<RafsiMap> = LazyLock::new(|| {
    serde_json::from_str(include_str!("cmavo_rafsi_list.json"))
        .expect("Failed to parse cmavo_rafsi_list.json")
});

static CMAVO_RAFSI_LIST_EXP: LazyLock<RafsiMap> = LazyLock::new(|| {
    serde_json::from_str(include_str!("cmavo_rafsi_list_exp.json"))
        .expect("Failed to parse cmavo_rafsi_list_exp.json")
});

fn build_reverse(map: &RafsiMap) -> ReverseMap {
    let mut reverse = ReverseMap::new();
    for (selrafsi, list) in map {
        for rafsi in list {
            reverse
                .entry(rafsi.clone())
                .or_insert_with(|| selrafsi.clone());
        }
    }
    reverse
}

static GISMU_REVERSE: LazyLock<ReverseMap> = LazyLock::new(|| build_reverse(&GISMU_RAFSI_LIST));
static GISMU_REVERSE_EXP: LazyLock<ReverseMap> =
    LazyLock::new(|| build_reverse(&GISMU_RAFSI_LIST_EXP));
static CMAVO_REVERSE: LazyLock<ReverseMap> = LazyLock::new(|| build_reverse(&CMAVO_RAFSI_LIST));
static CMAVO_REVERSE_EXP: LazyLock<ReverseMap> =
    LazyLock::new(|| build_reverse(&CMAVO_RAFSI_LIST_EXP));

pub fn get_gismu_rafsi_list() -> &'static RafsiMap {
    &GISMU_RAFSI_LIST
}

pub fn get_gismu_rafsi_list_exp() -> &'static RafsiMap {
    &GISMU_RAFSI_LIST_EXP
}

pub fn get_cmavo_rafsi_list() -> &'static RafsiMap {
    &CMAVO_RAFSI_LIST
}

pub fn get_cmavo_rafsi_list_exp() -> &'static RafsiMap {
    &CMAVO_RAFSI_LIST_EXP
}

pub fn reverse_gismu(rafsi: &str) -> Option<&'static str> {
    GISMU_REVERSE.get(rafsi).map(String::as_str)
}

pub fn reverse_gismu_exp(rafsi: &str) -> Option<&'static str> {
    GISMU_REVERSE_EXP.get(rafsi).map(String::as_str)
}

pub fn reverse_cmavo(rafsi: &str) -> Option<&'static str> {
    CMAVO_REVERSE.get(rafsi).map(String::as_str)
}

pub fn reverse_cmavo_exp(rafsi: &str) -> Option<&'static str> {
    CMAVO_REVERSE_EXP.get(rafsi).map(String::as_str)
}
