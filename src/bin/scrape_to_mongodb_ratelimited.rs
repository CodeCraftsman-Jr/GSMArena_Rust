use gsmarena_scraper::{fetch_all_brands, fetch_phones_by_brand, MongoDBClient, PhoneDocument};
use gsmarena_scraper::mongodb::parse_specifications;
use gsmarena;
use serde_json;
use std::error::Error;
use chrono::Utc;

/// Fetch phone specifications with retry logic
fn fetch_with_retry(phone_id: &str, max_retries: u32, retry_delay_ms: u64) -> Result<gsmarena::DeviceSpecification, String> {
    for attempt in 1..=max_retries {
        match std::panic::catch_unwind(|| gsmarena::get_specification(phone_id)) {
            Ok(spec) => return Ok(spec),
            Err(_) => {
                if attempt < max_retries {
                    eprintln!("    Retry {}/{} for {} after {}ms", attempt, max_retries, phone_id, retry_delay_ms);
                    std::thread::sleep(std::time::Duration::from_millis(retry_delay_ms * attempt as u64));
                }
            }
        }
    }
    Err(format!("Failed after {} retries", max_retries))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("GSMArena Scraper - MongoDB Integration (Rate Limited)");
    println!("=====================================================\n");

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
            .unwrap_or(usize::MAX)
    };

    let phones_per_brand = if args.len() > 2 {
        args[2].parse::<usize>().unwrap_or(usize::MAX)
    } else {
        std::env::var("PHONES_PER_BRAND")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(usize::MAX)
    };

    let collection_name = std::env::var("COLLECTION_NAME")
        .unwrap_or_else(|_| "gsmarena_phones".to_string());

    let skip_existing = std::env::var("SKIP_EXISTING")
        .unwrap_or_else(|_| "true".to_string())
        .parse::<bool>()
        .unwrap_or(true);

    let delay_between_phones = std::env::var("DELAY_BETWEEN_PHONES_MS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(500); // Default: 500ms delay between each phone

    let delay_between_brands = std::env::var("DELAY_BETWEEN_BRANDS_MS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(3000); // Default: 3 second delay between brands

    println!("Configuration:");
    println!("  Collection name: {}", collection_name);
    println!("  Max brands: {}", if max_brands == usize::MAX { "ALL".to_string() } else { max_brands.to_string() });
    println!("  Max phones per brand: {}", if phones_per_brand == usize::MAX { "ALL".to_string() } else { phones_per_brand.to_string() });
    println!("  Skip existing: {}", skip_existing);
    println!("  Delay between phones: {}ms (rate limiting)", delay_between_phones);
    println!("  Delay between brands: {}ms (rate limiting)", delay_between_brands);
    println!();

    // Connect to MongoDB
    println!("Connecting to MongoDB...");
    let mongo_client = MongoDBClient::from_env().await?;
    
    // Create indexes for better performance
    println!("Setting up database indexes...");
    mongo_client.create_indexes(&collection_name).await.ok(); // Ignore if already exists
    
    // Get initial count
    let initial_count = mongo_client.get_phone_count(&collection_name).await?;
    println!("Current phones in database: {}\n", initial_count);

    // Fetch all brands
    println!("Fetching brands from GSMArena...");
    let brands = fetch_all_brands()?;
    println!("✓ Found {} brands\n", brands.len());

    // Statistics
    let mut stats = Stats::default();

    // Process brands sequentially with rate limiting
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

        // Fetch and store specifications sequentially with rate limiting
        println!("  Fetching specifications (rate limited):");
        
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

            // Add delay before fetching to avoid rate limiting
            std::thread::sleep(std::time::Duration::from_millis(delay_between_phones));

            // Fetch specifications with retry logic
            let spec = match fetch_with_retry(&phone.phone_id, 3, 1000) {
                Ok(s) => s,
                Err(e) => {
                    println!(" ✗ Fetch error: {}", e);
                    stats.phones_failed += 1;
                    continue;
                }
            };
            
            // Convert to JSON
            let spec_json = match serde_json::to_value(&spec) {
                Ok(json) => json,
                Err(e) => {
                    println!(" ✗ JSON error: {}", e);
                    stats.phones_failed += 1;
                    continue;
                }
            };

            // Parse specifications into organized structure
            let (network, launch, body, display, platform, memory, main_camera, selfie_camera, 
                 sound, comms, features, battery, misc) = parse_specifications(&spec_json);

            let now = Utc::now();
            
            // Create phone document with organized data
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
        }

        println!();
        
        // Delay between brands to avoid rate limiting
        if brand_index + 1 < brands.len().min(max_brands) {
            println!("  ⏳ Waiting {}ms before next brand...", delay_between_brands);
            std::thread::sleep(std::time::Duration::from_millis(delay_between_brands));
            println!();
        }
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
