use gsmarena;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Serialize, Deserialize)]
struct PhoneSummary {
    name: String,
    brand: String,
    display: String,
    chipset: String,
    memory: String,
    camera: String,
    battery: String,
    price: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let brands = vec![
        "Samsung",
        "Apple",
        "Xiaomi",
        "OnePlus",
        "Google Pixel",
    ];

    println!("Scraping latest flagship phones from multiple brands\n");
    
    let mut summaries = Vec::new();

    for brand in brands {
        println!("Searching for {} phones...", brand);
        
        let results = gsmarena::search(brand).await?;
        
        if let Some(first) = results.first() {
            println!("  Found: {}", first.name);
            
            match gsmarena::phone(&first.url).await {
                Ok(phone) => {
                    let summary = PhoneSummary {
                        name: phone.name.clone(),
                        brand: brand.to_string(),
                        display: find_spec(&phone, "size").unwrap_or("N/A".to_string()),
                        chipset: find_spec(&phone, "chipset").unwrap_or("N/A".to_string()),
                        memory: find_spec(&phone, "memory").unwrap_or("N/A".to_string()),
                        camera: find_spec(&phone, "main camera").unwrap_or("N/A".to_string()),
                        battery: find_spec(&phone, "battery").unwrap_or("N/A".to_string()),
                        price: find_spec(&phone, "price").unwrap_or("N/A".to_string()),
                    };
                    summaries.push(summary);
                }
                Err(e) => eprintln!("  Error fetching details: {}", e),
            }
        }
        
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }

    println!("\n\n");
    println!("=".repeat(100));
    println!("MULTI-BRAND COMPARISON");
    println!("=".repeat(100));
    println!();

    for summary in &summaries {
        println!("Brand: {}", summary.brand);
        println!("Model: {}", summary.name);
        println!("Display: {}", summary.display);
        println!("Chipset: {}", summary.chipset);
        println!("Memory: {}", summary.memory);
        println!("Camera: {}", summary.camera);
        println!("Battery: {}", summary.battery);
        println!("Price: {}", summary.price);
        println!("{}", "-".repeat(100));
    }

    // Save to JSON
    let json = serde_json::to_string_pretty(&summaries)?;
    std::fs::write("multi_brand_comparison.json", json)?;
    println!("\n✓ Saved to 'multi_brand_comparison.json'");

    Ok(())
}

fn find_spec(phone: &gsmarena::Phone, key: &str) -> Option<String> {
    let search_key = key.to_lowercase();
    phone.specs.iter()
        .find(|(k, _)| k.to_lowercase().contains(&search_key))
        .map(|(_, v)| v.clone())
}
