use vlazba::{jvozba, jvokaha};

fn main() {
    // Generate lujvo candidates
    let results = jvozba::jvozba(
        &["klama".to_string(), "gasnu".to_string()], 
        false, 
        false,
        &jvozba::tools::RafsiOptions {
            exp_rafsi: true,
            custom_cmavo: None,
            custom_cmavo_exp: None,
            custom_gismu: None,
            custom_gismu_exp: None,
        }
    );
    
    println!("Top lujvo candidate: {}", results[0].lujvo);

    // Analyze existing lujvo
    match jvokaha::jvokaha("kalga'u") {
        Ok(parts) => println!("Decomposition: {:?}", parts),
        Err(e) => eprintln!("Error: {}", e),
    }
}
