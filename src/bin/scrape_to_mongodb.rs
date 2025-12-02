use gsmarena_scraper::{fetch_all_brands, fetch_phones_by_brand, MongoDBClient, PhoneDocument};
use gsmarena_scraper::mongodb::parse_specifications;
use gsmarena;
use serde_json;
use std::error::Error;
use chrono::Utc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("GSMArena Scraper - MongoDB Integration");
    println!("======================================\n");

    // Load environment variables from .env file (if it exists)
    dotenv::dotenv().ok();

    // Get configuration from environment variables or command line arguments
    let args: Vec<String> = std::env::args().collect();
    
    let max_brands = if args.len() > 1 {
        args[1].parse::<usize>().unwrap_or(usize::MAX)
    } else {
        std::env::var("MAX_BRANDS")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(usize::MAX) // Default: scrape all brands
    };

    let phones_per_brand = if args.len() > 2 {
        args[2].parse::<usize>().unwrap_or(usize::MAX)
    } else {
        std::env::var("PHONES_PER_BRAND")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(usize::MAX) // Default: all phones per brand
    };

    let collection_name = std::env::var("COLLECTION_NAME")
        .unwrap_or_else(|_| "gsmarena_phones".to_string());

    let skip_existing = std::env::var("SKIP_EXISTING")
        .unwrap_or_else(|_| "true".to_string())
        .parse::<bool>()
        .unwrap_or(true);

    println!("Configuration:");
    println!("  Collection name: {}", collection_name);
    println!("  Max brands: {}", if max_brands == usize::MAX { "ALL".to_string() } else { max_brands.to_string() });
    println!("  Max phones per brand: {}", if phones_per_brand == usize::MAX { "ALL".to_string() } else { phones_per_brand.to_string() });
    println!("  Skip existing: {}", skip_existing);
    println!();

    // Connect to MongoDB
    println!("Connecting to MongoDB...");
    let mongo_client = MongoDBClient::from_env().await?;
    
    // Get initial count
    let initial_count = mongo_client.get_phone_count(&collection_name).await?;
    println!("Current phones in database: {}\n", initial_count);

    // Fetch all brands
    println!("Fetching brands from GSMArena...");
    let brands = fetch_all_brands()?;
    println!("✓ Found {} brands\n", brands.len());

    let mut stats = Stats::default();

    // Process each brand
    for (brand_index, brand) in brands.iter().take(max_brands).enumerate() {
        println!("[{}/{}] Processing: {} ({} devices)", 
                 brand_index + 1, 
                 max_brands.min(brands.len()), 
                 brand.name,
                 brand.device_count);
        println!("{}", "-".repeat(70));

        // Fetch phone list for this brand
        print!("  Fetching phone list... ");
        let phones = match fetch_phones_by_brand(&brand.slug) {
            Ok(p) => {
                println!("✓ Found {} phones", p.len());
                p
            }
            Err(e) => {
                println!("✗ Error: {}", e);
                stats.brands_failed += 1;
                continue;
            }
        };

        stats.brands_processed += 1;
        stats.total_phones_found += phones.len();

        // Fetch and store specifications
        println!("  Fetching specifications:");
        for (phone_index, phone) in phones.iter().take(phones_per_brand).enumerate() {
            let display_index = phone_index + 1;
            let display_total = phones_per_brand.min(phones.len());
            
            print!("    [{}/{}] {}", display_index, display_total, phone.name);

            // Check if phone already exists
            if skip_existing {
                match mongo_client.phone_exists(&collection_name, &phone.phone_id).await {
                    Ok(true) => {
                        println!(" - Already exists, skipping");
                        stats.phones_skipped += 1;
                        continue;
                    }
                    Ok(false) => {}
                    Err(e) => {
                        println!(" - Error checking existence: {}", e);
                        stats.phones_failed += 1;
                        continue;
                    }
                }
            }

            // Fetch specifications
            let spec = gsmarena::get_specification(&phone.phone_id);
            
            // Convert to JSON
            let spec_json = match serde_json::to_value(&spec) {
                Ok(json) => json,
                Err(e) => {
                    println!(" ✗ Error converting to JSON: {}", e);
                    stats.phones_failed += 1;
                    continue;
                }
            };

            // Parse specifications into organized structure
            let (network, launch, body, display, platform, memory, main_camera, selfie_camera, 
                 sound, comms, features, battery, misc) = parse_specifications(&spec_json);

            let now = Utc::now();
            
            // Create phone document
            let phone_doc = PhoneDocument {
                phone_id: phone.phone_id.clone(),
                name: phone.name.clone(),
                brand: brand.name.clone(),
                url: phone.url.clone(),
                image_url: phone.image_url.clone(),
                source: "gsmarena".to_string(),
                network,
                launch,
                body,
                display,
                platform,
                memory,
                main_camera,
                selfie_camera,
                sound,
                comms,
                features,
                battery,
                misc,
                specifications_raw: spec_json,
                scraped_at: now,
                updated_at: now,
                version: 1,
            };

            // Insert into MongoDB
            match mongo_client.upsert_phone(&collection_name, phone_doc).await {
                Ok(_) => {
                    println!(" ✓");
                    stats.phones_inserted += 1;
                }
                Err(e) => {
                    println!(" ✗ Error inserting to MongoDB: {}", e);
                    stats.phones_failed += 1;
                }
            }

            // Small delay between requests to be respectful
            std::thread::sleep(std::time::Duration::from_millis(300));
        }

        println!();
    }

    // Final summary
    let final_count = mongo_client.get_phone_count(&collection_name).await?;
    
    println!("{}", "=".repeat(70));
    println!("✓ Scraping Complete!");
    println!("{}", "=".repeat(70));
    println!("Statistics:");
    println!("  Brands processed: {}/{}", stats.brands_processed, brands.len().min(max_brands));
    println!("  Brands failed: {}", stats.brands_failed);
    println!("  Total phones found: {}", stats.total_phones_found);
    println!("  Phones inserted/updated: {}", stats.phones_inserted);
    println!("  Phones skipped (existing): {}", stats.phones_skipped);
    println!("  Phones failed: {}", stats.phones_failed);
    println!("\nDatabase:");
    println!("  Collection: {}", collection_name);
    println!("  Previous count: {}", initial_count);
    println!("  Current count: {}", final_count);
    println!("  Net change: +{}", final_count as i64 - initial_count as i64);
    println!("{}", "=".repeat(70));

    Ok(())
}

#[derive(Default)]
struct Stats {
    brands_processed: usize,
    brands_failed: usize,
    total_phones_found: usize,
    phones_inserted: usize,
    phones_skipped: usize,
    phones_failed: usize,
}
