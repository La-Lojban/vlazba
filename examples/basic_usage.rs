use vlazba::{jvozba, jvokaha};

fn main() {
    // Generate lujvo candidates
    let results = jvozba::jvozba(
        &["klama".to_string(), "gasnu".to_string()], 
        false, 
        false,
        true
    );
    
    println!("Top lujvo candidate: {}", results[0].lujvo);

    // Analyze existing lujvo
    match jvokaha::jvokaha("kalga'u") {
        Ok(parts) => println!("Decomposition: {:?}", parts),
        Err(e) => eprintln!("Error: {}", e),
    }
}
