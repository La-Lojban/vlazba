use super::constants::language_weights;

fn is_weight_preset(weights_str: &str) -> bool {
    (weights_str.len() == 4 && weights_str.bytes().all(|b| b.is_ascii_digit()))
        || weights_str == "finprims"
}

pub fn generate_weights(weights_str: &str) -> anyhow::Result<Vec<f32>> {
    if is_weight_preset(weights_str) {
        language_weights()
            .get(weights_str)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("No weights registered for {weights_str}"))
    } else {
        weights_str
            .split(',')
            .map(|x| x.trim().parse::<f32>())
            .collect::<Result<Vec<f32>, _>>()
            .map_err(|_| anyhow::anyhow!("Values for weights must be numbers greater than zero"))
    }
}

pub fn validate_words(words: &[String], weights: &[f32]) -> anyhow::Result<()> {
    if words.len() != weights.len() {
        anyhow::bail!("Expected {} words as input", weights.len());
    }
    if words.iter().any(|word| word.len() < 2) {
        anyhow::bail!("Input words must be at least two letters long");
    }
    Ok(())
}
