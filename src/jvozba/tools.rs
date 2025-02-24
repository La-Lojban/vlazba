use serde_json;

use super::rafsi_list::{
    get_cmavo_rafsi_list, get_cmavo_rafsi_list_exp, get_gismu_rafsi_list, get_gismu_rafsi_list_exp,
};

pub fn create_every_possibility<T: Clone + serde::Serialize + for<'de> serde::Deserialize<'de>>(
    aa: Vec<Vec<T>>,
) -> Vec<Vec<T>> {
    let mut arr_arr: Vec<Vec<T>> =
        serde_json::from_str(&serde_json::to_string(&aa).unwrap()).unwrap();
    if arr_arr.is_empty() {
        return vec![vec![]];
    }
    let arr = arr_arr.pop().unwrap();

    let mut result: Vec<Vec<T>> = Vec::new();
    for e in arr {
        let sub_results = create_every_possibility(arr_arr.clone());
        for mut f in sub_results {
            f.push(e.clone());
            result.push(f);
        }
    }
    result
}

pub fn gismu_rafsi_list(a: &str, exp_rafsi: bool) -> Option<Vec<String>> {
    if let Some(rafsi) = get_gismu_rafsi_list().get(a) {
        if !rafsi.is_empty() {
            return Some(rafsi.clone());
        }
    }

    if exp_rafsi {
        if let Some(rafsi) = get_gismu_rafsi_list_exp().get(a) {
            if !rafsi.is_empty() {
                return Some(rafsi.clone());
            }
        }
    }
    Some(Vec::<String>::new())
}

pub fn cmavo_rafsi_list(a: &str, exp_rafsi: bool) -> Option<Vec<String>> {
    if let Some(rafsi) = get_cmavo_rafsi_list().get(a) {
        if !rafsi.is_empty() {
            return Some(rafsi.clone());
        }
    }

    if exp_rafsi {
        if let Some(rafsi) = get_cmavo_rafsi_list_exp().get(a) {
            if !rafsi.is_empty() {
                return Some(rafsi.clone());
            }
        }
    }
    None
}

pub fn get_candid(selrafsi: &str, is_last: bool, exp_rafsi: bool) -> Vec<String> {
    if let Some(a) = cmavo_rafsi_list(selrafsi, exp_rafsi) {
        return a;
    }
    if let Some(b) = gismu_rafsi_list(selrafsi, exp_rafsi) {
        let gismu = selrafsi;
        let mut candid = b;

        if is_last {
            candid.push(gismu.to_string());
        }

        let chopped = gismu[..gismu.len() - 1].to_string();
        if chopped != "brod" {
            candid.push(chopped);
        }
        candid
    } else {
        Vec::new()
    }
}

pub fn search_selrafsi_from_rafsi2(rafsi: &str, exp_rafsi: bool) -> Option<String> {
    if let Some(rafsis) = gismu_rafsi_list(rafsi, exp_rafsi) {
        if !rafsis.is_empty() {
            return Some(rafsi.to_owned());
        }
    }

    if rafsi != "brod" && rafsi.len() == 4 && !rafsi.contains('\'') {
        for vowel in "aeiou".chars() {
            let gismu_candid = format!("{}{}", rafsi, vowel);
            if gismu_rafsi_list(&gismu_candid, exp_rafsi).is_some() {
                return Some(gismu_candid);
            }
        }
    }

    for (i, rafsi_list) in get_gismu_rafsi_list().iter() {
        if rafsi_list.contains(&rafsi.to_string()) {
            return Some(i.clone());
        }
    }

    for (j, rafsi_list) in get_cmavo_rafsi_list().iter() {
        if rafsi_list.contains(&rafsi.to_string()) {
            return Some(j.clone());
        }
    }

    if exp_rafsi {
        for (i, rafsi_list) in get_gismu_rafsi_list_exp().iter() {
            if rafsi_list.contains(&rafsi.to_string()) {
                return Some(i.clone());
            }
        }

        for (j, rafsi_list) in get_cmavo_rafsi_list_exp().iter() {
            if rafsi_list.contains(&rafsi.to_string()) {
                return Some(j.clone());
            }
        }
    }

    None
}
