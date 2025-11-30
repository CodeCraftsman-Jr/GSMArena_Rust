use gsmarena_scraper::{fetch_all_brands, fetch_phones_by_brand, MongoDBClient, PhoneDocument};
use gsmarena_scraper::mongodb::parse_specifications;
use gsmarena;
use serde_json;
use std::error::Error;
use chrono::Utc;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use rayon::prelude::*;

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
    println!("GSMArena Scraper - MongoDB Integration (Parallel)");
    println!("==================================================\n");

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

    let parallel_threads = std::env::var("PARALLEL_THREADS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(4); // Default: 4 parallel threads (reduced for rate limiting)

    let delay_between_phones = std::env::var("DELAY_BETWEEN_PHONES_MS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(500); // Default: 500ms delay between each phone

    let delay_between_brands = std::env::var("DELAY_BETWEEN_BRANDS_MS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(2000); // Default: 2 second delay between brands

    println!("Configuration:");
    println!("  Collection name: {}", collection_name);
    println!("  Max brands: {}", if max_brands == usize::MAX { "ALL".to_string() } else { max_brands.to_string() });
    println!("  Max phones per brand: {}", if phones_per_brand == usize::MAX { "ALL".to_string() } else { phones_per_brand.to_string() });
    println!("  Skip existing: {}", skip_existing);
    println!("  Parallel threads: {}", parallel_threads);
    println!("  Delay between phones: {}ms", delay_between_phones);
    println!("  Delay between brands: {}ms", delay_between_brands);
    println!();

    // Connect to MongoDB
    println!("Connecting to MongoDB...");
    let mongo_client = Arc::new(MongoDBClient::from_env().await?);
    
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

    // Setup thread pool
    rayon::ThreadPoolBuilder::new()
        .num_threads(parallel_threads)
        .build_global()
        .unwrap();

    // Shared statistics
    let stats = Arc::new(Mutex::new(Stats::default()));
    let processed_count = Arc::new(AtomicUsize::new(0));

    // Process brands in parallel
    let brands_to_process: Vec<_> = brands.iter().take(max_brands).collect();
    let total_brands = brands_to_process.len();

    // Create a Tokio runtime handle for async operations in parallel threads
    let runtime = tokio::runtime::Runtime::new()?;
    let runtime_handle = runtime.handle().clone();

    // Process brands sequentially to avoid overwhelming the server
    for (brand_idx, brand) in brands_to_process.iter().enumerate() {
        let brand_num = brand_idx + 1;
        let handle = runtime_handle.clone();
        
        println!("[{}/{}] Processing: {} ({} devices)", 
                 brand_num, total_brands, brand.name, brand.device_count);
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
                let mut stats_lock = stats.lock().unwrap();
                stats_lock.brands_failed += 1;
                return;
            }
        };

        {
            let mut stats_lock = stats.lock().unwrap();
            stats_lock.brands_processed += 1;
            stats_lock.total_phones_found += phones.len();
        }

        // Fetch and store specifications in parallel
        println!("  Fetching specifications (parallel with rate limiting):");
        
        let phone_results: Vec<_> = phones.iter()
            .take(phones_per_brand)
            .enumerate()
            .collect::<Vec<_>>()
            .chunks(parallel_threads) // Process in batches to control parallelism
            .flat_map(|chunk| {
                let batch_results: Vec<_> = chunk.par_iter()
                    .map(|(phone_index, phone)| {
                        let display_index = phone_index + 1;
                        let display_total = phones_per_brand.min(phones.len());
                        
                        // Check if phone already exists (use blocking check)
                        if skip_existing {
                            let mongo_clone = Arc::clone(&mongo_client);
                            let collection_clone = collection_name.clone();
                            let phone_id_clone = phone.phone_id.clone();
                            
                            let exists = handle.block_on(async {
                                mongo_clone.phone_exists(&collection_clone, &phone_id_clone).await
                            });

                            match exists {
                                Ok(true) => {
                                    return PhoneResult::Skipped(format!("[{}/{}] {} - Already exists", 
                                        display_index, display_total, phone.name));
                                }
                                Ok(false) => {}
                                Err(e) => {
                                    return PhoneResult::Failed(format!("[{}/{}] {} - Error checking: {}", 
                                        display_index, display_total, phone.name, e));
                                }
                            }
                        }

                        // Add delay before fetching to avoid rate limiting
                        std::thread::sleep(std::time::Duration::from_millis(delay_between_phones));

                        // Fetch specifications with retry logic
                        let spec = match fetch_with_retry(&phone.phone_id, 3, 1000) {
                            Ok(s) => s,
                            Err(e) => {
                                return PhoneResult::Failed(format!("[{}/{}] {} - Fetch error: {}", 
                                    display_index, display_total, phone.name, e));
                            }
                        };
                        
                        // Convert to JSON
                        let spec_json = match serde_json::to_value(&spec) {
                            Ok(json) => json,
                            Err(e) => {
                                return PhoneResult::Failed(format!("[{}/{}] {} - JSON error: {}", 
                                    display_index, display_total, phone.name, e));
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
                        let mongo_clone = Arc::clone(&mongo_client);
                        let collection_clone = collection_name.clone();
                        
                        let insert_result = handle.block_on(async {
                            mongo_clone.upsert_phone(&collection_clone, phone_doc).await
                        });

                        match insert_result {
                            Ok(_) => PhoneResult::Success(format!("[{}/{}] {} ✓", 
                                display_index, display_total, phone.name)),
                            Err(e) => PhoneResult::Failed(format!("[{}/{}] {} - Insert error: {}", 
                                display_index, display_total, phone.name, e)),
                        }
                    })
                    .collect();
                
                // Small delay between batches
                std::thread::sleep(std::time::Duration::from_millis(200));
                batch_results
            })
            .collect();

        // Process results and update stats
        for result in phone_results {
            match result {
                PhoneResult::Success(msg) => {
                    println!("    {}", msg);
                    let mut stats_lock = stats.lock().unwrap();
                    stats_lock.phones_inserted += 1;
                }
                PhoneResult::Skipped(msg) => {
                    println!("    {}", msg);
                    let mut stats_lock = stats.lock().unwrap();
                    stats_lock.phones_skipped += 1;
                }
                PhoneResult::Failed(msg) => {
                    println!("    {}", msg);
                    let mut stats_lock = stats.lock().unwrap();
                    stats_lock.phones_failed += 1;
                }
            }
        }

        println!();
        
        // Delay between brands to avoid rate limiting
        if brand_num < total_brands {
            println!("  Waiting {}ms before next brand...", delay_between_brands);
            std::thread::sleep(std::time::Duration::from_millis(delay_between_brands));
        }
    }

    // Final summary
    let final_count = mongo_client.get_phone_count(&collection_name).await?;
    let stats_final = stats.lock().unwrap();
    
    println!("{}", "=".repeat(70));
    println!("✓ Scraping Complete!");
    println!("{}", "=".repeat(70));
    println!("Statistics:");
    println!("  Brands processed: {}/{}", stats_final.brands_processed, brands.len().min(max_brands));
    println!("  Brands failed: {}", stats_final.brands_failed);
    println!("  Total phones found: {}", stats_final.total_phones_found);
    println!("  Phones inserted/updated: {}", stats_final.phones_inserted);
    println!("  Phones skipped (existing): {}", stats_final.phones_skipped);
    println!("  Phones failed: {}", stats_final.phones_failed);
    println!("\nDatabase:");
    println!("  Collection: {}", collection_name);
    println!("  Previous count: {}", initial_count);
    println!("  Current count: {}", final_count);
    println!("  Net change: +{}", final_count as i64 - initial_count as i64);
    println!("{}", "=".repeat(70));

    // Properly shutdown the runtime
    drop(runtime);

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

enum PhoneResult {
    Success(String),
    Skipped(String),
    Failed(String),
}
