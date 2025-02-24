pub fn get_lujvo_score(rafsi_ynr_sequence: &[String]) -> i32 {
    let lujvo = rafsi_ynr_sequence.join("");
    let l = lujvo.len() as i32;
    let a = lujvo.matches('\'').count() as i32;
    let mut h = 0;
    let mut r = 0;

    for rafsi in rafsi_ynr_sequence {
        match get_cv_info(rafsi).as_str() {
            "C" | "Y" => h += 1, // ynr-hyphen
            "CVCCV" => r += 1,
            "CVCC" => r += 2,
            "CCVCV" => r += 3,
            "CCVC" => r += 4,
            "CVC" => r += 5,
            "CV'V" => r += 6,
            "CCV" => r += 7,
            "CVV" => r += 8,
            _ => {}
        }
    }

    let v = lujvo.chars().filter(|&c| "aeiou".contains(c)).count() as i32;
    
    (1000 * l) - (500 * a) + (100 * h) - (10 * r) - v
}

pub fn get_cv_info(v: &str) -> String {
    v.chars()
        .map(|c| match c {
            'a' | 'e' | 'i' | 'o' | 'u' => "V",
            'b' | 'c' | 'd' | 'f' | 'g' | 'j' | 'k' | 'l' | 'm' | 'n' | 'p' | 'r' | 's' | 't' | 'v' | 'x' | 'z' => "C",
            '\'' => "'",
            'y' => "Y",
            _ => "", // Skip unexpected characters
        })
        .collect()
}
