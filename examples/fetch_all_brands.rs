use gsmarena_scraper::{fetch_all_brands, fetch_phones_by_brand};
use serde_json;
use std::error::Error;
use std::fs;

fn main() -> Result<(), Box<dyn Error>> {
    println!("GSMArena - Fetch All Brands and Phones");
    println!("======================================\n");

    // Fetch all brands
    println!("Fetching all brands...");
    let brands = fetch_all_brands()?;
    println!("✓ Found {} brands\n", brands.len());

    // Save brands list
    let brands_json = serde_json::to_string_pretty(&brands)?;
    fs::write("all_brands.json", brands_json)?;
    println!("✓ Saved brands to 'all_brands.json'\n");

    // Show brands
    println!("Top 20 Brands:");
    println!("{:-<60}", "");
    for (i, brand) in brands.iter().take(20).enumerate() {
        println!("{:2}. {:30} ({:4} devices)", 
                 i + 1, brand.name, brand.device_count);
    }
    println!();

    // Fetch phones for each brand
    println!("\nFetching all phones from all brands...");
    println!("This will take a while. Results will be saved to 'all_phones_by_brand.json'\n");

    let mut all_phones_data = serde_json::Map::new();
    let mut total_phones = 0;

    for (index, brand) in brands.iter().enumerate() {
        println!("[{}/{}] Fetching: {}", index + 1, brands.len(), brand.name);
        
        match fetch_phones_by_brand(&brand.slug) {
            Ok(phones) => {
                println!("  ✓ Found {} phones", phones.len());
                total_phones += phones.len();
                
                let phones_json = serde_json::to_value(&phones)?;
                all_phones_data.insert(brand.name.clone(), phones_json);
                
                // Save progress periodically
                if (index + 1) % 10 == 0 {
                    let json = serde_json::to_string_pretty(&all_phones_data)?;
                    fs::write("all_phones_by_brand.json", json)?;
                    println!("  → Progress saved ({} brands, {} phones)", 
                             all_phones_data.len(), total_phones);
                }
            }
            Err(e) => {
                eprintln!("  ✗ Error: {}", e);
            }
        }
        
        // Delay between brands to be respectful
        std::thread::sleep(std::time::Duration::from_millis(500));
    }

    // Final save
    let json = serde_json::to_string_pretty(&all_phones_data)?;
    fs::write("all_phones_by_brand.json", json)?;
    
    println!("\n{}", "=".repeat(60));
    println!("✓ Complete!");
    println!("  Total brands: {}", brands.len());
    println!("  Total phones: {}", total_phones);
    println!("  Saved to: all_phones_by_brand.json");
    println!("{}", "=".repeat(60));

    Ok(())
}
