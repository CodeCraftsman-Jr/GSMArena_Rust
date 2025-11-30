use gsmarena;
use serde_json;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    let brand = if args.len() > 1 {
        args[1].clone()
    } else {
        "Apple".to_string()
    };

    println!("Scraping phones from brand: {}\n", brand);
    
    // Search for the brand
    let results = gsmarena::search(&brand).await?;
    
    println!("Found {} phones from {}\n", results.len(), brand);
    
    let mut all_phones = Vec::new();
    
    for (index, result) in results.iter().enumerate().take(10) {
        println!("[{}/{}] Fetching: {}", index + 1, results.len().min(10), result.name);
        
        match gsmarena::phone(&result.url).await {
            Ok(phone) => {
                all_phones.push(phone);
            }
            Err(e) => {
                eprintln!("  ✗ Error: {}", e);
            }
        }
        
        // Add a small delay to be respectful to the server
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }

    println!("\n✓ Successfully scraped {} phones", all_phones.len());

    // Save all phones to JSON
    let filename = format!("{}_phones.json", brand.to_lowercase());
    let json = serde_json::to_string_pretty(&all_phones)?;
    std::fs::write(&filename, json)?;
    println!("✓ Saved to '{}'", filename);

    // Print summary
    println!("\nSummary:");
    for phone in &all_phones {
        println!("  - {}", phone.name);
    }

    Ok(())
}
