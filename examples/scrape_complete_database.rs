use gsmarena_scraper::{fetch_all_brands, fetch_phones_by_brand};
use gsmarena;
use serde_json;
use std::error::Error;
use std::fs::{self, File};
use std::io::Write;

fn main() -> Result<(), Box<dyn Error>> {
    println!("GSMArena - Complete Database Scraper");
    println!("====================================\n");

    // Get command line arguments
    let args: Vec<String> = std::env::args().collect();
    let max_brands = if args.len() > 1 {
        args[1].parse::<usize>().unwrap_or(5)
    } else {
        5 // Default: scrape only 5 brands for testing
    };

    let phones_per_brand = if args.len() > 2 {
        args[2].parse::<usize>().unwrap_or(10)
    } else {
        10 // Default: 10 phones per brand
    };

    println!("Configuration:");
    println!("  Max brands to scrape: {}", max_brands);
    println!("  Max phones per brand: {}", phones_per_brand);
    println!();

    // Create output directory
    fs::create_dir_all("scraped_data")?;

    // Fetch all brands
    println!("Step 1: Fetching all brands...");
    let brands = fetch_all_brands()?;
    println!("✓ Found {} brands\n", brands.len());

    // Save brands list
    let brands_json = serde_json::to_string_pretty(&brands)?;
    fs::write("scraped_data/all_brands.json", brands_json)?;

    let mut stats = serde_json::Map::new();
    let mut total_phones_scraped = 0;
    let mut total_specs_fetched = 0;

    // Fetch phones and specs for each brand
    for (brand_index, brand) in brands.iter().take(max_brands).enumerate() {
        println!("\n[{}/{}] Processing Brand: {}", 
                 brand_index + 1, max_brands, brand.name);
        println!("{}", "-".repeat(60));

        // Fetch phone list for this brand
        println!("  Fetching phone list...");
        let phones = match fetch_phones_by_brand(&brand.slug) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("  ✗ Error fetching phones: {}", e);
                continue;
            }
        };
        
        println!("  ✓ Found {} phones", phones.len());
        total_phones_scraped += phones.len();

        // Save phone list for this brand
        let brand_dir = format!("scraped_data/{}", sanitize_filename(&brand.name));
        fs::create_dir_all(&brand_dir)?;
        
        let phones_json = serde_json::to_string_pretty(&phones)?;
        fs::write(format!("{}/phone_list.json", brand_dir), phones_json)?;

        // Fetch detailed specs for phones
        println!("  Fetching detailed specifications...");
        let mut brand_specs = Vec::new();

        for (phone_index, phone) in phones.iter().take(phones_per_brand).enumerate() {
            print!("    [{}/{}] {}", 
                   phone_index + 1, 
                   phones_per_brand.min(phones.len()), 
                   phone.name);
            
            match gsmarena::get_specification(&phone.phone_id) {
                spec => {
                    println!(" ✓");
                    
                    // Save individual phone spec
                    let spec_json = serde_json::to_string_pretty(&spec)?;
                    let filename = format!("{}/{}.json", brand_dir, sanitize_filename(&phone.phone_id));
                    fs::write(filename, spec_json)?;
                    
                    brand_specs.push(spec);
                    total_specs_fetched += 1;
                }
            }
            
            // Small delay between requests
            std::thread::sleep(std::time::Duration::from_millis(300));
        }

        // Save all specs for this brand
        let brand_specs_json = serde_json::to_string_pretty(&brand_specs)?;
        fs::write(format!("{}/all_specs.json", brand_dir), brand_specs_json)?;

        println!("  ✓ Saved {} specifications", brand_specs.len());

        // Update stats
        stats.insert(brand.name.clone(), serde_json::json!({
            "total_phones": phones.len(),
            "specs_fetched": brand_specs.len(),
            "directory": brand_dir
        }));

        // Save progress
        let stats_json = serde_json::to_string_pretty(&stats)?;
        fs::write("scraped_data/scraping_stats.json", stats_json)?;
    }

    // Final summary
    println!("\n{}", "=".repeat(60));
    println!("✓ Scraping Complete!");
    println!("{}", "=".repeat(60));
    println!("Statistics:");
    println!("  Brands processed: {}", stats.len());
    println!("  Total phones found: {}", total_phones_scraped);
    println!("  Specifications fetched: {}", total_specs_fetched);
    println!("\nOutput directory: scraped_data/");
    println!("  - all_brands.json: List of all brands");
    println!("  - [brand_name]/phone_list.json: Phone list per brand");
    println!("  - [brand_name]/[phone_id].json: Individual phone specs");
    println!("  - [brand_name]/all_specs.json: All specs for brand");
    println!("  - scraping_stats.json: Scraping statistics");
    println!("{}", "=".repeat(60));

    Ok(())
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect()
}
