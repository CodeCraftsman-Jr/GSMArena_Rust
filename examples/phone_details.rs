use gsmarena;
use serde_json;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    let phone_query = if args.len() > 1 {
        args[1..].join(" ")
    } else {
        "iPhone 15 Pro".to_string()
    };

    println!("Fetching details for: {}\n", phone_query);
    
    // First search for the phone
    let search_results = gsmarena::search(&phone_query).await?;
    
    if search_results.is_empty() {
        println!("No phones found matching '{}'", phone_query);
        return Ok(());
    }

    let first_result = &search_results[0];
    println!("Found: {}", first_result.name);
    println!("Fetching detailed specifications...\n");

    // Get detailed information
    let phone = gsmarena::phone(&first_result.url).await?;

    println!("=== {} ===\n", phone.name);
    if let Some(img) = &phone.img {
        println!("Image: {}\n", img);
    }

    // Display specifications
    println!("Specifications:");
    println!("-".repeat(50));
    for (key, value) in &phone.specs {
        println!("{:25} : {}", key, value);
    }

    // Save to JSON
    let filename = format!("{}.json", phone.name.replace(" ", "_"));
    let json = serde_json::to_string_pretty(&phone)?;
    std::fs::write(&filename, json)?;
    println!("\n✓ Saved to '{}'", filename);

    Ok(())
}
